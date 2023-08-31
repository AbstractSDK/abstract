use crate::{
    contract::ChallengeApp,
    state::{ChallengeEntry, ChallengeEntryUpdate, CheckIn, Friend, UpdateFriendsOpKind, Vote},
};
use abstract_core::objects::AssetEntry;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Uint128};
use croncat_app::croncat_integration_utils::CronCatInterval;

abstract_app::app_msg_types!(ChallengeApp, ChallengeExecuteMsg, ChallengeQueryMsg);

#[cosmwasm_schema::cw_serde]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    EveryNBlocks(u64),
}

impl Frequency {
    pub fn to_interval(self) -> CronCatInterval {
        match self {
            Frequency::EveryNBlocks(blocks) => CronCatInterval::Block(blocks),
            Frequency::Daily => unimplemented!(),
            Frequency::Weekly => unimplemented!(),
            Frequency::Monthly => unimplemented!(),
        }
    }
}

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    /// Native gas/stake asset for this chain
    pub native_asset: AssetEntry,
    /// Amount in native coins to forfeit when a challenge is lost
    pub forfeit_amount: Uint128,
}

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
        metadata: Option<String>,
    },
    CastVote {
        challenge_id: u64,
        vote: Vote<String>,
    },
    CountVotes {
        challenge_id: u64,
    },
    VetoVote {
        voter: String,
        challenge_id: u64,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ChallengeQueryMsg {
    #[returns(ChallengeResponse)]
    Challenge { challenge_id: u64 },
    #[returns(FriendsResponse)]
    Friends { challenge_id: u64 },
    #[returns(CheckInResponse)]
    CheckIn { challenge_id: u64 },
    #[returns(VotesResponse)]
    Votes { challenge_id: u64 },
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeResponse {
    pub challenge: Option<ChallengeEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct CheckInResponse {
    pub check_in: Option<CheckIn>,
}

#[cosmwasm_schema::cw_serde]
pub struct FriendsResponse(pub Option<Vec<Friend<Addr>>>);

#[cosmwasm_schema::cw_serde]
pub struct VotesResponse(pub Option<Vec<Vote<Addr>>>);
