use abstract_core::objects::AssetEntry;
use cosmwasm_std::{Addr, Deps, StdResult, Uint128};
use cw_address_like::AddressLike;
use cw_storage_plus::{Item, Map};
use cw_utils::Expiration;

use crate::msg::ChallengeRequest;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub native_denom: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntry {
    pub name: String,
    pub strike_asset: AssetEntry,
    pub strike_strategy: StrikeStrategy,
    pub description: String,
    pub end: Expiration,
    pub status: ChallengeStatus,
    pub admin_strikes: AdminStrikes,
}

/// Strategy for striking the admin
#[cosmwasm_schema::cw_serde]
pub enum StrikeStrategy {
    /// Split amount between friends
    Split(Uint128),
    /// Amount for every friend
    PerFriend(Uint128),
}

#[cosmwasm_schema::cw_serde]
pub struct AdminStrikes {
    /// The number of strikes the admin has incurred.
    pub num_strikes: u8,
    /// When num_strikes reached the limit, the challenge will be cancelled.
    pub limit: u8,
}

impl AdminStrikes {
    fn new(limit: Option<u8>) -> Self {
        AdminStrikes {
            num_strikes: 0,
            // One-time strike by default
            limit: limit.unwrap_or(1),
        }
    }
}

impl ChallengeEntry {
    /// Creates a new challenge entry with the default status of Uninitialized and no admin strikes.
    pub fn new(request: ChallengeRequest) -> Self {
        ChallengeEntry {
            name: request.name,
            strike_asset: request.strike_asset,
            strike_strategy: request.strike_strategy,
            description: request.description,
            end: request.end,
            status: ChallengeStatus::default(),
            admin_strikes: AdminStrikes::new(request.strikes_limit),
        }
    }
}

/// The status of a challenge. This can be used to trigger an automated Croncat job
/// based on the value of the status
#[derive(Default)]
#[cosmwasm_schema::cw_serde]
pub enum ChallengeStatus {
    /// The challenge is active and can be voted on.
    #[default]
    Active,
    /// The challenge was cancelled and no collateral was paid out.
    Cancelled,
    /// The challenge has pased the end time.
    Over,
}

/// Only this struct and these fields are allowed to be updated.
/// The status cannot be externally updated, it is updated by the contract.
#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntryUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub enum UpdateFriendsOpKind {
    Add,
    Remove,
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
    /// The address of the voter
    pub voter: T,
    /// The vote result
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
    pub fn optimistic(self) -> Vote<Addr> {
        Vote {
            voter: self.voter,
            approval: Some(self.approval.unwrap_or(true)),
        }
    }
}

pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const CHALLENGE_LIST: Map<u64, ChallengeEntry> = Map::new("challenge_list");
pub const CHALLENGE_FRIENDS: Map<u64, Vec<Friend<Addr>>> = Map::new("challenge_friends");

/// Key is a tuple of (challenge_id, check_in.last_checked_in, voter_address).
/// By using a composite key, it ensures only one user can vote per check_in.
pub const VOTES: Map<(u64, Addr), Vote<Addr>> = Map::new("votes");

/// For looking up all the votes by challenge_id. This is used to tally the votes.
pub const CHALLENGE_VOTES: Map<u64, Vec<Vote<Addr>>> = Map::new("challenge_votes");
