use abstract_core::app;
use abstract_sdk::base::{ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint};
use cosmwasm_schema::QueryResponses;

use crate::contract::App;

/// Abstract App instantiate msg
pub type InstantiateMsg = <App as InstantiateEndpoint>::InstantiateMsg;
pub type ExecuteMsg = <App as ExecuteEndpoint>::ExecuteMsg;
pub type QueryMsg = <App as QueryEndpoint>::QueryMsg;
pub type MigrateMsg = <App as MigrateEndpoint>::MigrateMsg;

impl app::AppExecuteMsg for AppExecuteMsg {}
impl app::AppQueryMsg for AppQueryMsg {}

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum AppExecuteMsg {
    UpdateConfig {},
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum AppQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub enum AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum Cw20HookMsg {
    Deposit {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {}
