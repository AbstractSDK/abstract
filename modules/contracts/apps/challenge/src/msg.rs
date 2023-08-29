use crate::{
    contract::ChallengeApp,
    state::{ChallengeEntry, Friend},
};
use abstract_core::objects::AssetEntry;
use abstract_dex_adapter::msg::OfferAsset;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Uint128;
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
    UpdateConfig {
        new_native_denom: Option<String>,
        new_forfeit_amount: Option<Uint128>,
    },
    CreateChallenge {
        challenge: ChallengeEntry,
    },
    UpdateChallenge {
        challenge_id: String,
        challenge: ChallengeEntry,
    },
    CancelChallenge {
        challenge_id: String,
    },
    AddFriendForChallenge {
        challenge_id: String,
        friend_address: String,
        friend_name: String,
    },
    RemoveFriendForChallenge {
        challenge_id: String,
        friend_address: String,
    },
    AddFriendsForChallenge {
        challenge_id: String,
        friends: Vec<Friend>,
    },
    DailyCheckIn {
        challenge_id: String,
    },
    CastVote {
        challenge_id: String,
        vote: Option<bool>,
    },
    CountVotes {
        challenge_id: String,
    },
    ChargePenalty {
        challenge_id: String,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ChallengeQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ChallengeResponse)]
    Challenge { challenge_id: String },
    #[returns(FriendResponse)]
    Friend {
        challenge_id: String,
        friend_address: String,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub native_asset: AssetEntry,
    pub forfeit_amount: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct ChallengeResponse {
    pub challenge: Option<ChallengeEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct FriendResponse {
    pub friend: Option<Friend>,
}
