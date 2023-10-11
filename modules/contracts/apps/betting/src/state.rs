use crate::contract::{BetApp, BetResult};
use crate::error::BetError;
use crate::handlers::query;
use crate::handlers::query::get_total_bets_for_team;
use crate::msg::RoundResponse;
use abstract_core::objects::ans_host::AnsHost;
use abstract_core::objects::fee::Fee;
use abstract_core::objects::validation::{validate_description, validate_name};
use abstract_core::objects::{AccountId, AnsAsset, AssetEntry};
use abstract_core::AbstractResult;
use abstract_sdk::features::AbstractNameService;
use abstract_sdk::{AccountingInterface, Resolve};
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Order, StdError, StdResult, Storage, Uint128};
use cw_asset::{Asset, AssetInfo};
use cw_storage_plus::{Item, Map, MultiIndex};
use std::collections::{HashMap, HashSet};

/// State stores LP token address
/// BaseState is initialized in contract
#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub rake: Fee,
}

impl Config {
    pub fn validate(&self, _deps: Deps) -> BetResult<()> {
        Ok(())
    }
}

pub const DEFAULT_RAKE_PERCENT: u64 = 10;

pub type RoundId = u64;

#[cosmwasm_schema::cw_serde]
pub struct RoundInfo {
    pub name: String,
    pub description: String,
    pub bet_asset: AssetEntry,
    pub status: RoundStatus,
}

#[cosmwasm_schema::cw_serde]
pub enum RoundStatus {
    Open,
    /// Round is closed, and there may or may not be a winning team
    Closed {
        winning_team: Option<AccountId>,
    },
    RewardsDistributed,
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
        let info = ROUNDS
            .load(storage, self.id())
            .map_err(|_| BetError::RoundNotFound(self.id()))?;
        Ok(info)
    }
    pub fn status(&self, storage: &dyn Storage) -> BetResult<RoundStatus> {
        let info = self.info(storage)?;
        Ok(info.status)
    }

    pub fn accounts(&self, storage: &dyn Storage) -> BetResult<Vec<AccountId>> {
        Ok(ROUNDS_TO_ACCOUNTS.load(storage, self.id())?)
    }

    pub fn bets(&self, storage: &dyn Storage) -> BetResult<Vec<(Addr, Uint128)>> {
        let all_bets = BETS
            .prefix(self.id())
            .range(storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        let bets = all_bets
            .into_iter()
            .map(|(_, value)| value)
            .flatten()
            .collect::<Vec<(Addr, Uint128)>>();

        Ok(bets)
    }

    fn bet_count(&self, storage: &dyn Storage) -> BetResult<u128> {
        let all_keys: Vec<_> = BETS
            .prefix(self.id())
            .keys(storage, None, None, Order::Ascending)
            .collect();
        Ok(all_keys.len() as u128)
    }

    pub fn set_status(&self, storage: &mut dyn Storage, status: RoundStatus) -> BetResult<()> {
        let mut info = self.info(storage)?;
        info.status = status;
        ROUNDS.save(storage, self.id(), &info)?;
        Ok(())
    }

    pub fn assert_not_closed(&self, storage: &dyn Storage) -> BetResult<()> {
        let info = self.status(storage)?;
        if matches!(info, RoundStatus::Closed { .. }) {
            return Err(BetError::RoundAlreadyClosed(self.id()));
        }
        Ok(())
    }

    pub fn total_bet(&self, storage: &dyn Storage) -> BetResult<Uint128> {
        let total = query::get_total_bets_for_all_accounts(storage, self.id())?;
        Ok(total)
    }

    pub fn query(&self, deps: Deps) -> BetResult<RoundResponse> {
        let info = self.info(deps.storage)?;
        let accounts = self.accounts(deps.storage)?;
        let bet_count = self.bet_count(deps.storage)?;
        let total_bet = self.total_bet(deps.storage)?;

        Ok(RoundResponse {
            id: self.id(),
            name: info.name,
            description: info.description,
            teams: accounts,
            status: info.status,
            bet_count,
            total_bet: AnsAsset {
                name: info.bet_asset,
                amount: total_bet,
            },
        })
    }

    /// Register accounts to a round and error out if duplicates are found.
    /// *unchecked* for account existence.
    pub fn update_accounts(
        &self,
        deps: DepsMut,
        to_add: Vec<AccountOdds>,
        to_remove: Vec<AccountId>,
    ) -> BetResult<()> {
        // Load existing accounts associated with the round
        let mut existing_accounts: Vec<AccountId> = ROUNDS_TO_ACCOUNTS
            .may_load(deps.storage, self.id())?
            .unwrap_or_default();
        let rake = CONFIG.load(deps.storage)?.rake.share();

        deps.api.debug(&format!("rake: {:?}", rake));

        // Add new account IDs after checking for duplicates
        for AccountOdds { account_id, odds } in to_add.into_iter() {
            if existing_accounts.contains(&account_id.clone()) {
                return Err(StdError::generic_err(format!(
                    "Duplicate Account ID found: {}",
                    account_id
                ))
                .into());
            }
            existing_accounts.push(account_id.clone());
            deps.api.debug(&format!(
                "odds {:?} / Decimal::one() + rake.clone() {:?}",
                odds,
                Decimal::one() + rake.clone()
            ));
            let mut edged_odds = odds.checked_div(Decimal::one() + rake.clone())?;
            // Don't allow odds to go below 1
            if edged_odds < Decimal::one() {
                edged_odds = Decimal::one();
            }
            ODDS.save(deps.storage, (self.id(), account_id), &edged_odds)?;
        }

        // Remove specified account IDs
        for account_id in to_remove.into_iter() {
            if let Some(index) = existing_accounts.iter().position(|x| *x == account_id) {
                existing_accounts.remove(index);
            }
            ODDS.remove(deps.storage, (self.id(), account_id.clone()));
        }

        // Save the updated list of accounts back to storage
        ROUNDS_TO_ACCOUNTS.save(deps.storage, self.id(), &existing_accounts)?;

        Ok(())
    }
}

impl RoundInfo {
    pub fn validate(&self, deps: Deps, ans_host: &AnsHost) -> BetResult<()> {
        validate_name(self.name.as_str())?;
        validate_description(Some(self.description.as_str()))?;
        // TODO: could save the resolved asset in storage to avoid querying every time
        self.bet_asset.resolve(&deps.querier, ans_host)?;
        Ok(())
    }
}

pub type RoundTeamKey = (RoundId, AccountId);

pub type OddsType = Decimal; // Represents odds with two decimal precision

/// TODO: remove round ID and replace this tuple
#[cosmwasm_schema::cw_serde]
pub struct Bet {
    pub account_id: AccountId,
    pub asset: AnsAsset,
}

impl Bet {
    pub fn validate(&self, deps: Deps, round: &Round) -> BetResult<()> {
        // check that the account being bet on is registered
        round.assert_not_closed(deps.storage)?;

        if self.asset.amount.is_zero() {
            return Err(BetError::InvalidBet {});
        }

        let bet_asset = round.info(deps.storage)?.bet_asset;

        // ensure that the asset matches the base asset
        if self.asset.name != bet_asset {
            return Err(BetError::DepositAssetNotBase(self.asset.name.to_string()));
        }

        let accounts = round.accounts(deps.storage)?;
        let bet_account_id = &self.account_id;

        if !accounts.contains(bet_account_id) {
            return Err(BetError::AccountNotParticipating {
                account_id: bet_account_id.clone(),
                round_id: round.id(),
            });
        }

        Ok(())
    }
}

#[cosmwasm_schema::cw_serde]
pub struct AccountOdds {
    pub account_id: AccountId,
    pub odds: OddsType, // e.g., 250 for 2.50 odds
}

// Map that stores all the rounds and their information
pub const ROUNDS: Map<RoundId, RoundInfo> = Map::new("rounds");

pub const ROUNDS_TO_ACCOUNTS: Map<RoundId, Vec<AccountId>> = Map::new("round_teams");
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const BETS: Map<RoundTeamKey, Vec<(Addr, Uint128)>> = Map::new("bets");
pub const ODDS: Map<RoundTeamKey, OddsType> = Map::new("odds");
