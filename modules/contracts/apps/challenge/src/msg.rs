use crate::{contract::ChallengeApp, state::ChallengeEntry};
use abstract_core::objects::AssetEntry;
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
    /// Task balance threshold to trigger refill, put it at zero if you consider to never refill your tasks
    pub refill_threshold: Uint128,
}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum ChallengeExecuteMsg {
    //@Todo: Add ExecuteMsgs
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum ChallengeQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ChallengeResponse)]
    Acc { acc_id: String },
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
