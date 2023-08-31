use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_std::{Addr, Deps, StdResult};
use cw_address_like::AddressLike;
use cw_storage_plus::{Item, Map};

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntry {
    pub name: String,
    pub collateral: Penalty,
    pub description: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntryUpdate {
    pub name: Option<String>,
    pub collateral: Option<Penalty>,
    pub description: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub enum UpdateFriendsOpKind {
    Add,
    Remove,
}

#[cosmwasm_schema::cw_serde]
pub enum Penalty {
    FixedAmount {
        asset: OfferAsset,
    },
    Daily {
        asset: OfferAsset,
        split_between_friends: bool,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct Friend<T: AddressLike> {
    pub address: T,
    pub name: String,
}

impl Friend<String> {
    /// A helper to convert from a string address to an Addr.
    /// Additionally it validates the address.
    pub fn check(self, deps: Deps) -> StdResult<Friend<Addr>> {
        Ok(Friend {
            address: deps.api.addr_validate(&self.address)?,
            name: self.name,
        })
    }
}

#[cosmwasm_schema::cw_serde]
pub struct Vote<T: AddressLike> {
    pub voter: T,
    pub approval: Option<bool>,
}

impl Vote<String> {
    /// A helper to convert from a string address to an Addr.
    /// Additionally it validates the address.
    pub fn check(self, deps: Deps) -> StdResult<Vote<Addr>> {
        Ok(Vote {
            voter: deps.api.addr_validate(&self.voter)?,
            approval: self.approval,
        })
    }
}

impl Vote<Addr> {
    /// If the vote approval field is None, we assume the voter approves,
    /// and return a vote with the approval field set to Some(true).
    pub fn optimisitc(self) -> Vote<Addr> {
        Vote {
            voter: self.voter,
            approval: Some(self.approval.unwrap_or(true)),
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct CheckIn {
    pub last_checked_in: u64, // blockheight
    pub next_check_in_by: u64,
    pub metadata: Option<String>,
}

pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const ADMIN: Item<Addr> = Item::new("admin");
pub const CHALLENGE_LIST: Map<u64, ChallengeEntry> = Map::new("challenge_list");
pub const CHALLENGE_FRIENDS: Map<u64, Vec<Friend<Addr>>> = Map::new("challenge_friends");
/// Key is a tuple of (challenge_id, voter_address).
pub const VOTES: Map<(u64, Addr), Vote<Addr>> = Map::new("votes");
// use a snapshot map?
pub const DAILY_CHECK_INS: Map<u64, CheckIn> = Map::new("daily_checkins");
