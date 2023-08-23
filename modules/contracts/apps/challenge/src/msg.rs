use crate::{contract::AccApp, state::AccEntry};
use abstract_core::objects::PoolReference;
use cosmwasm_std::{Decimal, Uint128};

abstract_app::app_msg_types!(AccApp, AccExecuteMsg, AccQueryMsg);

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
    /// Amount in native coins for accountability creation task and refill amount
    pub forfeit_creation_amount: Uint128,
    /// Task balance threshold to trigger refill, put it at zero if you consider to never refill your tasks
    pub refill_threshold: Uint128,
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum AccQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AccResponse)]
    Acc { acc_id: String },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub native_asset: AssetEntry,
    pub forfeit_amount: Uint128,
    pub refill_threshold: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct AccResponse {
    pub acc: Option<AccEntry>,
    pub pool_references: Vec<PoolReference>,
}
