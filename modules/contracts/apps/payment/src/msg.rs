use abstract_core::objects::{AnsAsset, AssetEntry, DexName};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Uint128};

use crate::contract::PaymentApp;

// This is used for type safety
// The second part is used to indicate the messages are used as the apps messages
// This is equivalent to
// pub type InstantiateMsg = <PaymentApp as abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
// pub type ExecuteMsg = <PaymentApp as abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
// pub type QueryMsg = <PaymentApp as abstract_sdk::base::QueryEndpoint>::QueryMsg;
// pub type MigrateMsg = <PaymentApp as abstract_sdk::base::MigrateEndpoint>::MigrateMsg;

// impl app::AppExecuteMsg for AppExecuteMsg {}
// impl app::AppQueryMsg for AppQueryMsg {}
abstract_app::app_msg_types!(PaymentApp, AppExecuteMsg, AppQueryMsg);

/// PaymentApp instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    pub desired_asset: Option<AssetEntry>,
    pub denom_asset: String,
    pub exchanges: Vec<DexName>,
}

/// PaymentApp execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum AppExecuteMsg {
    #[cfg_attr(feature = "interface", payable)]
    Tip {},
    UpdateConfig {
        // TODO: Clearable #ABS-269
        desired_asset: Option<AssetEntry>,
        denom_asset: Option<String>,
        exchanges: Option<Vec<DexName>>,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum AppQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(TipperResponse)]
    Tipper {
        address: String,
        start_after: Option<AssetEntry>,
        limit: Option<u32>,
    },
    #[returns(TipCountResponse)]
    TipCount {},
    #[returns(TippersCountResponse)]
    ListTippersCount {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns tipped amount of given asset at block height
    #[returns(TipAmountAtHeightResponse)]
    TipAtHeight {
        address: String,
        asset: AssetEntry,
        height: u64,
    },
}

#[cosmwasm_schema::cw_serde]
pub enum AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct Cw20TipMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub desired_asset: Option<AssetEntry>,
    pub denom_asset: String,
    pub exchanges: Vec<DexName>,
}

#[cosmwasm_schema::cw_serde]
pub struct TipperResponse {
    pub address: Addr,
    pub tip_count: u32,
    pub total_amounts: Vec<AnsAsset>,
}

#[cosmwasm_schema::cw_serde]
pub struct TipperCountResponse {
    pub address: Addr,
    pub count: u32,
}

#[cosmwasm_schema::cw_serde]
pub struct TippersCountResponse {
    pub tippers: Vec<TipperCountResponse>,
}

#[cosmwasm_schema::cw_serde]
pub struct TipCountResponse {
    pub count: u32,
}

#[cosmwasm_schema::cw_serde]
pub struct TipAmountAtHeightResponse {
    pub amount: Option<Uint128>,
}
