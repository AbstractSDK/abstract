use abstract_core::objects::{AssetEntry, DexName};
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
abstract_app::app_messages!(PaymentApp, AppExecuteMsg, AppQueryMsg);

/// PaymentApp instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {
    pub desired_asset: Option<AssetEntry>,
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
    Tipper { address: String },
    #[returns(TipCountResponse)]
    TipCount {},
    #[returns(TippersResponse)]
    ListTippers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cosmwasm_schema::cw_serde]
pub enum AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct Cw20TipMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub desired_asset: Option<AssetEntry>,
    pub exchanges: Vec<DexName>,
}

#[cosmwasm_schema::cw_serde]
pub struct TipperResponse {
    pub address: Addr,
    pub total_amount: Uint128,
    pub count: u32,
}

#[cosmwasm_schema::cw_serde]
pub struct TippersResponse {
    pub tippers: Vec<TipperResponse>,
}

#[cosmwasm_schema::cw_serde]
pub struct TipCountResponse {
    pub count: u32,
}
