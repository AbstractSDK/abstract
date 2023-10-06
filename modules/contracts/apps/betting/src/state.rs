use std::collections::{HashMap, HashSet};
use abstract_core::AbstractResult;
use abstract_core::objects::fee::Fee;
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Order, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map, MultiIndex};
use abstract_core::objects::{AccountId, AnsAsset, AssetEntry};
use abstract_core::objects::ans_host::AnsHost;
use abstract_core::objects::validation::{validate_description, validate_name};
use abstract_sdk::{AccountingInterface, Resolve};
use abstract_sdk::features::AbstractNameService;
use cw_asset::{Asset, AssetInfo};
use crate::contract::{BetApp, BetResult};
use crate::error::BetError;
use crate::handlers::query::get_total_bets_for_account;
use crate::msg::RoundResponse;

/// State stores LP token address
/// BaseState is initialized in contract
#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub rake: Fee,
    pub bet_asset: AssetEntry,
}

impl Config {
    pub fn validate(&self, deps: Deps, app: &BetApp) -> BetResult<()> {
        // ensure that the base betting token exists
        let ans_host = app.ans_host(deps)?;
        self.bet_asset.resolve(&deps.querier, &ans_host)?;

        Ok(())
    }
}

pub const DEFAULT_RAKE_PERCENT: u64 = 10;


pub type RoundId = u64;

#[cosmwasm_schema::cw_serde]
pub struct RoundInfo {
    pub name: String,
    pub description: String,
    pub base_bet_token: AssetEntry,
}

#[derive(Default)]
#[cosmwasm_schema::cw_serde]
pub struct State {
  pub next_round_id: RoundId,
}

pub struct Round(pub RoundId);
impl Round {
    pub fn new(id: RoundId) -> Self {
        Round(id)
    }

    pub fn id(&self) -> RoundId {
        self.0
    }

    pub fn info(&self, storage: &dyn Storage) -> BetResult<RoundInfo> {
        let info = ROUNDS.load(storage, self.id()).map_err(|_| BetError::RoundNotFound(self.id()))?;
        Ok(info)
    }
    pub fn accounts(&self, storage: &dyn Storage) -> BetResult<Vec<AccountId>> {
        Ok(ROUND_ACCOUNTS.load(storage, self.id())?)
    }

    fn bet_count(&self, storage: &dyn Storage) -> BetResult<u128> {
        let all_keys: Vec<_> = BETS.prefix(self.id()).keys(storage, None, None, Order::Ascending).collect();
        Ok(all_keys.len() as u128)
    }

    pub fn query(&self, deps: Deps) -> BetResult<RoundResponse> {
        let info = self.info(deps.storage)?;
        let accounts = self.accounts(deps.storage)?;
        let total_bets = self.bet_count(deps.storage)?;
        Ok(RoundResponse {
            id: self.id(),
            name: info.name,
            description: info.description,
            teams: accounts.into_iter().map(|x| (self.id(), x)).collect(),
            // TODO
            winning_team: None,
            total_bets,
        })
    }

    pub fn total_bets(&self, storage: &dyn Storage) -> BetResult<Uint128> {
        let accounts = self.accounts(storage)?;
        let total: Uint128 = accounts.iter().map(|account_id| {
            get_total_bets_for_account(storage, self.id(), account_id.clone()).unwrap_or_default()
        }).sum();
        Ok(total)
    }

    /// Register accounts to a round and error out if duplicates are found.
    /// *unchecked* for account existence.
    pub fn update_accounts(&self, deps: DepsMut, to_add: Vec<AccountOdds>, to_remove: Vec<AccountId>) -> BetResult<()> {
        // Load existing accounts associated with the round
        let mut existing_accounts: Vec<AccountId> = ROUND_ACCOUNTS.may_load(deps.storage, self.id())?.unwrap_or_default();

        // Add new account IDs after checking for duplicates
        for AccountOdds {account_id, odds } in to_add.into_iter() {
            if existing_accounts.contains(&account_id.clone()) {
                return Err(StdError::generic_err(format!("Duplicate Account ID found: {}", account_id)).into());
            }
            existing_accounts.push(account_id.clone());
            ODDS.save(deps.storage, (self.id(), account_id), &odds)?;
        }

        // Remove specified account IDs
        for account_id in to_remove.into_iter() {
            if let Some(index) = existing_accounts.iter().position(|x| *x == account_id) {
                existing_accounts.remove(index);
            }
            ODDS.remove(deps.storage, (self.id(), account_id.clone()));
        }

        // Save the updated list of accounts back to storage
        ROUND_ACCOUNTS.save(deps.storage, self.id(), &existing_accounts)?;

        Ok(())
    }

}



impl RoundInfo {
    pub fn validate(&self, deps: Deps, ans_host: &AnsHost) -> BetResult<()> {
        validate_name(self.name.as_str())?;
        validate_description(Some(self.description.as_str()))?;
        // TODO: could save the resolved asset in storage to avoid querying every time
        self.base_bet_token.resolve(&deps.querier, ans_host)?;
        Ok(())
    }


}

pub type RoundTeam = (RoundId, AccountId);


#[cosmwasm_schema::cw_serde]

pub struct NewBet {
    pub round_id: RoundId,
    pub account_id: AccountId,
    pub asset: AnsAsset,
}

pub struct ValidatedBet {
    pub round: Round,
    pub account_id: AccountId,
    pub asset: Asset,
}

type OddsInt = Uint128;  // Represents odds with two decimal precision


impl NewBet {
    pub fn validate(&self, deps: Deps, base_asset: &AssetEntry) -> BetResult<()> {
        if self.asset.amount.is_zero() {
            return Err(BetError::InvalidFee {});
        }

        // ensure that the asset matches the base asset
        if &self.asset.name != base_asset {
            return Err(BetError::DepositAssetNotBase(self.asset.name.to_string()));
        }

        // check that the account being bet on is registered
        let round = Round::new(self.round_id);
        let accounts = round.accounts(deps.storage)?;
        let bet_account_id = &self.account_id;
        if !accounts.contains(bet_account_id) {
            return Err(BetError::AccountNotParticipating {
                account_id: bet_account_id.clone(),
                round_id: round.id()
            });
        }

        Ok(())
        //
        // Ok(ValidatedBet {
        //     round,
        //     account_id: self.account_id.clone(),
        //     asset: resolved_asset,
        // })
    }
}

#[cosmwasm_schema::cw_serde]
pub struct AccountOdds {
    pub account_id: AccountId,
    pub odds: OddsInt, // e.g., 250 for 2.50 odds
}

pub const ROUNDS: Map<RoundId, RoundInfo> = Map::new("rounds");

pub const ROUND_ACCOUNTS: Map<RoundId, Vec<AccountId>> = Map::new("round_teams");
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const BETS: Map<(RoundId, AccountId), Vec<(Addr, Uint128)>> = Map::new("bets");
pub const ODDS: Map<(RoundId, AccountId), OddsInt> = Map::new("odds");
