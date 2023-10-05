#![warn(missing_docs)]
//! Message types for the challenge app
use crate::{
    contract::ChallengeApp,
    state::{
        AdminStrikes, ChallengeEntry, ChallengeEntryUpdate, StrikeStrategy, UpdateFriendsOpKind,
    },
};
use abstract_core::objects::{
    voting::{Vote, VoteStatus},
    AssetEntry,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;
use cw_utils::{Duration, Expiration};

abstract_app::app_msg_types!(ChallengeApp, ChallengeExecuteMsg, ChallengeQueryMsg);

/// Challenge execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum ChallengeExecuteMsg {
    /// Create new challenge
    CreateChallenge {
        /// New challenge arguments
        challenge_req: ChallengeRequest,
    },
    /// Update existing challenge
    UpdateChallenge {
        /// Id of the challenge to update
        challenge_id: u64,
        /// Updates to this challenge
        challenge: ChallengeEntryUpdate,
    },
    /// Cancel challenge
    CancelChallenge {
        ///Challenge Id to cancel
        challenge_id: u64,
    },
    /// Update list of friends for challenge
    UpdateFriendsForChallenge {
        /// Id of the challenge to update
        challenge_id: u64,
        /// List of added or removed Friends
        friends: Vec<String>,
        /// Kind of operation: add or remove friends
        op_kind: UpdateFriendsOpKind,
    },
    /// Cast vote as a friend
    CastVote {
        /// Id of challenge to cast vote on
        challenge_id: u64,
        /// If the vote.approval is None, we assume the voter approves,
        /// and the contract will internally set the approval field to Some(true).
        /// This is because we assume that if a friend didn't vote, the friend approves,
        /// otherwise the voter would Vote with approval set to Some(false).
        vote: Vote,
    },
}

/// Challenge query messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ChallengeQueryMsg {
    /// Get challenge info, will return null if there was no challenge by Id
    #[returns(ChallengeResponse)]
    Challenge {
        /// Id of requested challenge
        challenge_id: u64,
    },
    /// Get list of challenges
    #[returns(ChallengesResponse)]
    Challenges {
        /// start after challenge Id
        start_after: u64,
        /// Max amount of challenges in response
        limit: u32,
    },
    /// List of friends by Id
    #[returns(FriendsResponse)]
    Friends {
        /// Id of requested challenge
        challenge_id: u64,
    },
    /// Get last vote of friend
    #[returns(VoteResponse)]
    Vote {
        /// Addr of the friend
        voter_addr: String,
        /// Id of requested challenge
        challenge_id: u64,
    },
}

/// Response for challenge query
#[cosmwasm_schema::cw_serde]
pub struct ChallengeResponse {
    /// Challenge info, will return null if there was no challenge by Id
    pub challenge: Option<ChallengeEntryResponse>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeEntryResponse {
    pub name: String,
    pub strike_asset: AssetEntry,
    pub strike_strategy: StrikeStrategy,
    pub description: String,
    pub end: Expiration,
    pub status: VoteStatus,
    pub admin_strikes: AdminStrikes,
}

/// Arguments for new challenge
#[cosmwasm_schema::cw_serde]
pub struct ChallengeRequest {
    /// Name of challenge
    pub name: String,
    /// Asset for punishment for failing a challenge
    pub strike_asset: AssetEntry,
    /// How strike will get distributed between friends
    pub strike_strategy: StrikeStrategy,
    /// Desciption of the challenge
    pub description: String,
    /// In what duration challenge should end
    pub duration: Duration,
    /// Strike limit, defaults to 1
    pub strikes_limit: Option<u8>,
    /// Initial list of friends
    pub init_friends: Vec<String>,
}

/// Response for vote query
#[cosmwasm_schema::cw_serde]
pub struct VoteResponse {
    /// The vote, will return null if there was no vote by this user
    pub vote: Option<Vote>,
}

/// Response for challenges query
/// Returns a list of challenges
#[cosmwasm_schema::cw_serde]
pub struct ChallengesResponse(pub Vec<ChallengeEntry>);

/// Response for friends query
/// Returns a list of friends
#[cosmwasm_schema::cw_serde]
pub struct FriendsResponse(pub Vec<Addr>);
