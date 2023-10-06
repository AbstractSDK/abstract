use std::collections::HashSet;
use abstract_core::AbstractResult;
use abstract_core::objects::fee::Fee;
use cosmwasm_std::{Addr, Decimal, Deps};
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
pub struct Track {
    pub name: String,
    pub description: String,
}

#[derive(Default)]
#[cosmwasm_schema::cw_serde]
pub struct State {
  pub next_track_id: TrackId,
}


impl Track {
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

pub const TRACKS: Map<TrackId, Track> = Map::new("tracks");

pub const TRACK_TEAMS: Map<TrackId, HashSet<AccountId>> = Map::new("track_teams");
pub const COTFIG_2: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
