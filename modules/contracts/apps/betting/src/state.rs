use std::collections::HashSet;
use abstract_core::AbstractResult;
use abstract_core::objects::fee::Fee;
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use abstract_core::objects::{AccountId, AnsAsset, AssetEntry};
use abstract_core::objects::ans_host::AnsHost;
use abstract_core::objects::validation::{validate_description, validate_name};
use abstract_sdk::{AccountingInterface, Resolve};
use abstract_sdk::features::AbstractNameService;
use cw_asset::{Asset, AssetInfo};
use crate::contract::{BetApp, BetResult};
use crate::error::BetError;

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


pub type TrackId = u64;

#[cosmwasm_schema::cw_serde]
pub struct TrackInfo {
    pub name: String,
    pub description: String,
    pub base_bet_token: AssetEntry,
}

#[derive(Default)]
#[cosmwasm_schema::cw_serde]
pub struct State {
  pub next_track_id: TrackId,
}

pub struct Track(pub TrackId);
impl Track {
    pub fn new(id: TrackId) -> Self {
        Track(id)
    }

    pub fn id(&self) -> TrackId {
        self.0
    }

    pub fn info(&self, deps: Deps) -> BetResult<TrackInfo> {
        let info = TRACKS.load(deps.storage, self.id()).map_err(|_| BetError::TrackNotFound(self.id()))?;
        Ok(info)
    }
    pub fn accounts(&self, storage: &dyn Storage) -> BetResult<Vec<AccountId>> {
        Ok(TRACK_ACCOUNTS.load(storage, self.id())?)
    }

    /// Register accounts to a track and error out if duplicates are found.
    /// *unchecked* for account existence.
    pub fn update_accounts(&self, deps: DepsMut, to_add: Vec<AccountId>, to_remove: Vec<AccountId>) -> BetResult<()> {
        // Load existing accounts associated with the track
        let mut existing_accounts: Vec<AccountId> = TRACK_ACCOUNTS.may_load(deps.storage, self.id())?.unwrap_or_default();

        // Add new account IDs after checking for duplicates
        for account_id in to_add.into_iter() {
            if existing_accounts.contains(&account_id) {
                return Err(StdError::generic_err(format!("Duplicate Account ID found: {}", account_id)).into());
            }
            existing_accounts.push(account_id);
        }

        // Remove specified account IDs
        for account_id in to_remove.into_iter() {
            if let Some(index) = existing_accounts.iter().position(|x| *x == account_id) {
                existing_accounts.remove(index);
            }
        }

        // Save the updated list of accounts back to storage
        TRACK_ACCOUNTS.save(deps.storage, self.id(), &existing_accounts)?;

        Ok(())
    }

}



impl TrackInfo {
    pub fn validate(&self, deps: Deps, ans_host: &AnsHost) -> BetResult<()> {
        validate_name(self.name.as_str())?;
        validate_description(Some(self.description.as_str()))?;
        // TODO: could save the resolved asset in storage to avoid querying every time
        self.base_bet_token.resolve(&deps.querier, ans_host)?;
        Ok(())
    }


}

pub type TrackTeam = (TrackId, AccountId);


#[cosmwasm_schema::cw_serde]

pub struct NewBet {
    pub track_id: TrackId,
    pub account_id: AccountId,
    pub asset: AnsAsset,
}

pub struct ValidatedBet {
    pub track: Track,
    pub account_id: AccountId,
    pub asset: Asset,
}

impl NewBet {
    pub fn validate(&self, deps: Deps, ans_host: &AnsHost, base_asset: &AssetEntry) -> BetResult<()> {
        if self.asset.amount.is_zero() {
            return Err(BetError::InvalidFee {});
        }

        // ensure that the asset matches the base asset
        if &self.asset.name != base_asset {
            return Err(BetError::DepositAssetNotBase(self.asset.name.to_string()));
        }

        // check that the account being bet on is registered
        let track = Track::new(self.track_id);
        let accounts = track.accounts(deps.storage)?;
        let bet_account_id = &self.account_id;
        if !accounts.contains(bet_account_id) {
            return Err(BetError::AccountNotParticipating {
                account_id: bet_account_id.clone(),
                track_id: track.id()
            });
        }

        Ok(())
        //
        // Ok(ValidatedBet {
        //     track,
        //     account_id: self.account_id.clone(),
        //     asset: resolved_asset,
        // })
    }
}

pub const TRACKS: Map<TrackId, TrackInfo> = Map::new("tracks");

pub const TRACK_ACCOUNTS: Map<TrackId, Vec<AccountId>> = Map::new("track_teams");
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
