use cosmwasm_schema::QueryResponses;

use crate::contract::App;

// This is used for type safety
// The second part is used to indicate the messages are used as the apps messages
// This is equivalent to
// pub type InstantiateMsg = <App as abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
// pub type ExecuteMsg = <App as abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
// pub type QueryMsg = <App as abstract_sdk::base::QueryEndpoint>::QueryMsg;
// pub type MigrateMsg = <App as abstract_sdk::base::MigrateEndpoint>::MigrateMsg;

// impl app::AppExecuteMsg for AppExecuteMsg {}
// impl app::AppQueryMsg for AppQueryMsg {}
abstract_app::app_messages!(App, AppExecuteMsg, AppQueryMsg);

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
pub struct ConfigResponse {}
