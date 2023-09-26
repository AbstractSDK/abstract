#![warn(missing_docs)]
//! Message types for the challenge app
use crate::{
    contract::ChallengeApp,
    state::{ChallengeEntry, ChallengeEntryUpdate, CheckIn, Friend, UpdateFriendsOpKind, Vote},
};
use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

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
        friends: Vec<Friend<String>>,
        /// Kind of operation: add or remove friends
        op_kind: UpdateFriendsOpKind,
    },
    /// Daily check in by the challenge author
    DailyCheckIn {
        /// Id of the challenge to check in
        challenge_id: u64,
        /// metadata can be added for extra description of the check-in.
        /// For example, if the check-in is a photo, the metadata can be a link to the photo.
        metadata: Option<String>,
    },
    /// Cast vote as a friend
    CastVote {
        /// Id of challenge to cast vote on
        challenge_id: u64,
        /// If the vote.approval is None, we assume the voter approves,
        /// and the contract will internally set the approval field to Some(true).
        /// This is because we assume that if a friend didn't vote, the friend approves,
        /// otherwise the voter would Vote with approval set to Some(false).
        vote: Vote<String>,
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
    /// List of check-ins by Id
    #[returns(CheckInsResponse)]
    CheckIns {
        /// Id of requested challenge
        challenge_id: u64,
    },
    /// Get last vote of friend
    #[returns(VoteResponse)]
    Vote {
        /// Block height of last check in
        last_check_in: u64,
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
    pub challenge: Option<ChallengeEntry>,
}

/// Arguments for new challenge
#[cosmwasm_schema::cw_serde]
pub struct ChallengeRequest {
    /// Name of challenge
    pub name: String,
    /// Asset punishment for failing a challenge
    pub collateral: OfferAsset,
    /// Desciption of the challenge
    pub description: String,
    /// In what period challenge should end
    pub end: DurationChoice,
}

/// Response for check_ins query
/// Returns a list of check ins
#[cosmwasm_schema::cw_serde]
pub struct CheckInsResponse(pub Vec<CheckIn>);

/// Duration for challenge
#[cosmwasm_schema::cw_serde]
pub enum DurationChoice {
    /// One week
    Week,
    /// One month
    Month,
    /// Quarter of the year
    Quarter,
    /// One year
    Year,
    /// 100 years
    OneHundredYears,
}

/// Response for vote query
#[cosmwasm_schema::cw_serde]
pub struct VoteResponse {
    /// The vote, will return null if there was no vote by this user
    pub vote: Option<Vote<Addr>>,
}

/// Response for challenges query
/// Returns a list of challenges
#[cosmwasm_schema::cw_serde]
pub struct ChallengesResponse(pub Vec<ChallengeEntry>);

/// Response for friends query
/// Returns a list of friends
#[cosmwasm_schema::cw_serde]
pub struct FriendsResponse(pub Vec<Friend<Addr>>);
