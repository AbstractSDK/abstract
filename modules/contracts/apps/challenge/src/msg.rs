use crate::{
    contract::ChallengeApp,
    state::{ChallengeEntry, ChallengeEntryUpdate, CheckIn, Friend, UpdateFriendsOpKind, Vote},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

abstract_app::app_msg_types!(ChallengeApp, ChallengeExecuteMsg, ChallengeQueryMsg);

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum ChallengeExecuteMsg {
    CreateChallenge {
        challenge: ChallengeEntry,
    },
    UpdateChallenge {
        challenge_id: u64,
        challenge: ChallengeEntryUpdate,
    },
    CancelChallenge {
        challenge_id: u64,
    },
    UpdateFriendsForChallenge {
        challenge_id: u64,
        friends: Vec<Friend<String>>,
        op_kind: UpdateFriendsOpKind,
    },
    DailyCheckIn {
        challenge_id: u64,
        /// metadata can be added for extra description of the check-in.
        /// For example, if the check-in is a photo, the metadata can be a link to the photo.
        metadata: Option<String>,
    },
    CastVote {
        challenge_id: u64,
        /// If the vote.approval is None, we assume the voter approves,
        /// and the contract will internally set the approval field to Some(true).
        /// This is because we assume that if a friend didn't vote, the friend approves,
        /// otherwise the voter would Vote with approval set to Some(false).
        vote: Vote<String>,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ChallengeQueryMsg {
    #[returns(ChallengeResponse)]
    Challenge { challenge_id: u64 },
    #[returns(ChallengesResponse)]
    Challenges { start_after: u64, limit: u32 },
    #[returns(FriendsResponse)]
    Friends { challenge_id: u64 },
    #[returns(CheckInsResponse)]
    CheckIns { challenge_id: u64 },
    #[returns(VoteResponse)]
    Vote {
        challenge_id: u64,
        voter_addr: String,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeResponse {
    pub challenge: Option<ChallengeEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct CheckInsResponse(pub Vec<CheckIn>);

#[cosmwasm_schema::cw_serde]
pub struct VoteResponse {
    pub vote: Option<Vote<Addr>>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengesResponse(pub Vec<ChallengeEntry>);

#[cosmwasm_schema::cw_serde]
pub struct FriendsResponse(pub Vec<Friend<Addr>>);
