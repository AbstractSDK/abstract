use abstract_app::std::objects::{
    validation::{self, ValidationError},
    voting::{ProposalId, SimpleVoting},
    AssetEntry,
};
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Item, Map};

use crate::msg::{ChallengeRequest, Friend};

pub const MAX_AMOUNT_OF_FRIENDS: u64 = 20;

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
    pub proposal_duration_seconds: Uint64,
    pub end_timestamp: Timestamp,
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
        self.num_strikes >= self.limit
    }
}

impl ChallengeEntry {
    /// Creates a new challenge entry with the default status of Uninitialized and no admin strikes.
    pub fn new(
        request: ChallengeRequest,
        end_timestamp: Timestamp,
    ) -> Result<Self, ValidationError> {
        // validate namd and description
        validation::validate_name(&request.name)?;
        validation::validate_description(request.description.as_deref())?;

        Ok(ChallengeEntry {
            name: request.name,
            strike_asset: request.strike_asset,
            strike_strategy: request.strike_strategy,
            description: request.description.unwrap_or_default(),
            admin_strikes: AdminStrikes::new(request.strikes_limit),
            proposal_duration_seconds: request.proposal_duration_seconds,
            end_timestamp,
        })
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

pub const CHALLENGES: Map<u64, ChallengeEntry> = Map::new("challenges");
/// Friends list for the challenge
// Reduces gas consumption to load all friends
// Helpful during distributing penalty and re-creation voting
pub const CHALLENGE_FRIENDS: Map<u64, Vec<Friend<Addr>>> = Map::new("friends");
pub const CHALLENGE_PROPOSALS: Map<(u64, ProposalId), cosmwasm_std::Empty> = Map::new("proposals");
