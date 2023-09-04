use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_std::{Addr, Deps, Env, StdResult};
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
    pub end_block: u64,
    pub status: ChallengeStatus,
    pub admin_strikes: [bool; 3],
}

impl ChallengeEntry {
    /// Creates a new challenge entry with the default status of Uninitialized and no admin strikes.
    pub fn new(name: String, collateral: Penalty, description: String, end_block: u64) -> Self {
        ChallengeEntry {
            name,
            collateral,
            description,
            end_block,
            status: ChallengeStatus::default(),
            admin_strikes: [false; 3],
        }
    }
}

/// The status of a challenge. This can be used to trigger an automated Croncat job
/// based on the value of the status
#[cosmwasm_schema::cw_serde]
pub enum ChallengeStatus {
    /// The challenge has not been initialized yet. This is the default state.
    Uninitialized,
    /// The challenge is active and can be voted on.
    Active,
    /// The challenge was cancelled and no collateral was paid out.
    Cancelled,
    /// The challenge is over, the votes have not yet been counted. count_votes needs to be called
    /// to determine the outcome.
    OverAndPending,
    /// The challenge is over, the admin has completed the challenge, the votes have been counted.
    /// The admin completed the challenge, so the collateral has remained with the owner.
    OverAndCompleted,
    /// The challenge is over, the votes have been conted and the admin has failed,
    /// their collateral is owed to the friends.
    /// This valued can be used to trigger an automated Croncat job to pay out the collateral.
    OverAndFailed,
}

impl Default for ChallengeStatus {
    fn default() -> Self {
        ChallengeStatus::Uninitialized
    }
}

/// Only this struct and these fields are allowed to be updated.
/// The status cannot be externally updated, it is updated by the contract.
#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntryUpdate {
    pub name: Option<String>,
    pub collateral: Option<Penalty>,
    pub description: Option<String>,
    pub end_block: Option<u64>,
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

/// The check in struct is used to track the admin's check ins.
#[cosmwasm_schema::cw_serde]
pub struct CheckIn {
    /// The blockheight of the last check in.
    pub last_checked_in: u64,
    /// The blockheight of the next check in.
    /// In the case of a missed check in, this will always be pushed forward
    /// internally by the contract.
    pub next_check_in_by: u64,
    pub metadata: Option<String>,
}

impl CheckIn {
    pub fn initial(env: &Env) -> Self {
        CheckIn {
            last_checked_in: env.block.height,
            next_check_in_by: env.block.height + 100_000,
            metadata: None,
        }
    }
}

pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const ADMIN: Item<Addr> = Item::new("admin");
pub const CHALLENGE_LIST: Map<u64, ChallengeEntry> = Map::new("challenge_list");
pub const CHALLENGE_FRIENDS: Map<u64, Vec<Friend<Addr>>> = Map::new("challenge_friends");

/// Key is a tuple of (challenge_id, voter_address).
/// By using a composite key, it ensures only one user can vote per challenge.
pub const VOTES: Map<(u64, Addr), Vote<Addr>> = Map::new("votes");

/// For looking up all the votes by id. This is used to tally the votes.
pub const CHALLENGE_VOTES: Map<u64, Vec<Vote<Addr>>> = Map::new("challenge_votes");

// use a snapshot map?
pub const DAILY_CHECK_INS: Map<u64, CheckIn> = Map::new("daily_checkins");
