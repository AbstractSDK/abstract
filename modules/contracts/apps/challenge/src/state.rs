use std::collections::HashSet;

use abstract_core::objects::{
    voting::{SimpleVoting, VoteId},
    AssetEntry,
};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

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
    pub admin_strikes: AdminStrikes,
    pub current_vote_id: VoteId,
    pub previous_vote_ids: Vec<VoteId>,
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

    pub fn strike(&mut self) -> bool {
        self.num_strikes += 1;
        // check if it's last strike
        let last_strike = self.num_strikes >= self.limit;
        last_strike
    }
}

impl ChallengeEntry {
    /// Creates a new challenge entry with the default status of Uninitialized and no admin strikes.
    pub fn new(request: ChallengeRequest, vote_id: VoteId) -> Self {
        ChallengeEntry {
            name: request.name,
            strike_asset: request.strike_asset,
            strike_strategy: request.strike_strategy,
            description: request.description,
            admin_strikes: AdminStrikes::new(request.strikes_limit),
            current_vote_id: vote_id,
            previous_vote_ids: Vec::default(),
        }
    }
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
    Add {},
    Remove {},
}

pub const NEXT_ID: Item<u64> = Item::new("next_id");
pub const SIMPLE_VOTING: SimpleVoting =
    SimpleVoting::new("votes", "votes_id", "votes_info", "votes_config");

pub const CHALLENGE_LIST: Map<u64, ChallengeEntry> = Map::new("challenge_list");
/// Friends list for the challenge
// Reduces gas consumption to load all friends
// Helpful during distributing penalty and re-creation voting
pub const FRIENDS_LIST: Map<u64, HashSet<Addr>> = Map::new("friends_list");
