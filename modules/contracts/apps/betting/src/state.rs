use std::collections::HashSet;
use abstract_core::AbstractResult;
use abstract_core::objects::fee::Fee;
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, StdResult};
use cw_storage_plus::{Item, Map};
use abstract_core::objects::{AccountId, AnsAsset, AssetEntry};
use abstract_core::objects::validation::{validate_description, validate_name};
use crate::contract::EtfResult;
use crate::error::BetError;

/// State stores LP token address
/// BaseState is initialized in contract
#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub rake: Fee,
}

const DEFAULT_RAKE_PERCENT: u64 = 10;

impl Default for Config {
    fn default() -> Self {
        Config {
            rake: Fee::new(Decimal::percent(DEFAULT_RAKE_PERCENT)).unwrap(),
        }
    }
}

pub type TrackId = u64;

#[cosmwasm_schema::cw_serde]
pub struct TrackInfo {
    pub name: String,
    pub description: String,
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

    pub fn info(&self, deps: Deps) -> EtfResult<TrackInfo> {
        let info = TRACKS.load(deps.storage, self.id()).map_err(|_| BetError::TrackNotFound(self.id()))?;
        Ok(info)
    }
    pub fn accounts(&self, deps: Deps) -> EtfResult<Vec<AccountId>> {
        Ok(TRACK_ACCOUNTS.load(deps.storage, self.id())?)
    }

    /// Register an account to a track without checking the account's existence
    pub fn unchecked_register_accounts(&self, deps: DepsMut, account_ids: Vec<AccountId>) -> EtfResult<()> {
        TRACK_ACCOUNTS.update(deps.storage, self.id(), |teams| -> StdResult<Vec<AccountId>> {
            let mut teams: Vec<AccountId> = teams.unwrap_or_default();
            teams.extend(account_ids);
            Ok(teams)
        })?;
        Ok(())
    }
}



impl TrackInfo {
    pub fn validate(&self) -> EtfResult<()> {
        validate_name(self.name.as_str())?;
        validate_description(Some(self.description.as_str()))?;
        Ok(())
    }


}

pub type TrackTeam = (TrackId, AccountId);

#[cosmwasm_schema::cw_serde]

pub struct NewBet {
    track_id: TrackId,
    account_id: AccountId,
    asset: AnsAsset,
}

impl NewBet {
    pub fn validate(&self, deps: Deps) -> EtfResult<()> {
        if self.asset.amount.is_zero() {
            return Err(BetError::InvalidFee {});
        }

        Ok(())
    }
}

pub const TRACKS: Map<TrackId, TrackInfo> = Map::new("tracks");

pub const TRACK_ACCOUNTS: Map<TrackId, Vec<AccountId>> = Map::new("track_teams");
pub const COTFIG_2: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
