use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{CosmosMsg, Decimal, Uint128};
use wyndex::asset::Asset;

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the factory contract
    pub factory: String,
    /// Owner of the creator (instantiator of the factory)
    pub owner: String,
    /// The asset to send to the voted-for lp staking contracts every epoch
    pub rewards_asset: Asset,
    pub epoch_length: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateRewards { amount: Uint128 },
}

#[cw_serde]
pub enum MigrateMsg {
    /// Used to instantiate from cw-placeholder
    Init(InstantiateMsg),
    Update {},
}

// Queries copied from gauge-orchestrator for now (we could use a common crate for this)
/// Queries the gauge requires from the adapter contract in order to function
#[cw_serde]
#[derive(QueryResponses)]
pub enum AdapterQueryMsg {
    #[returns(crate::state::Config)]
    Config {},
    #[returns(AllOptionsResponse)]
    AllOptions {},
    #[returns(CheckOptionResponse)]
    CheckOption { option: String },
    #[returns(SampleGaugeMsgsResponse)]
    SampleGaugeMsgs {
        /// option along with weight
        /// sum of all weights should be 1.0 (within rounding error)
        selected: Vec<(String, Decimal)>,
    },
}

#[cw_serde]
pub struct AllOptionsResponse {
    pub options: Vec<String>,
}

#[cw_serde]
pub struct CheckOptionResponse {
    pub valid: bool,
}

#[cw_serde]
pub struct SampleGaugeMsgsResponse {
    pub execute: Vec<CosmosMsg>,
}
