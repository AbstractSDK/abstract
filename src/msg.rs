use abstract_core::app;
use abstract_sdk::base::{ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint};
use cosmwasm_schema::QueryResponses;

use crate::contract::TemplateApp;

/// Abstract App instantiate msg
pub type InstantiateMsg = <TemplateApp as InstantiateEndpoint>::InstantiateMsg;
pub type ExecuteMsg = <TemplateApp as ExecuteEndpoint>::ExecuteMsg;
pub type QueryMsg = <TemplateApp as QueryEndpoint>::QueryMsg;
pub type MigrateMsg = <TemplateApp as MigrateEndpoint>::MigrateMsg;

impl app::AppExecuteMsg for TemplateExecuteMsg {}
impl app::AppQueryMsg for TemplateQueryMsg {}

/// Template instantiate message
#[cosmwasm_schema::cw_serde]
pub struct TemplateInstantiateMsg {}

/// Template execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum TemplateExecuteMsg {
    UpdateConfig {},
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum TemplateQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub enum TemplateMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum Cw20HookMsg {
    Deposit {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {}
