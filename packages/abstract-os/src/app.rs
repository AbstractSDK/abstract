//! # Abstract Add-On
//!
//! `abstract_os::app` implements shared functionality that's useful for creating new Abstract apps.
//!
//! ## Description
//! An app is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract while also being migratable.

use crate::base::{
    ExecuteMsg as MiddlewareExecMsg, InstantiateMsg as MiddlewareInstantiateMsg,
    MigrateMsg as MiddlewareMigrateMsg, QueryMsg as MiddlewareQueryMsg,
};

pub type ExecuteMsg<AppMsg, ReceiveMsg = Empty> = MiddlewareExecMsg<BaseExecuteMsg, AppMsg, ReceiveMsg>;
pub type QueryMsg<AppMsg = Empty> = MiddlewareQueryMsg<BaseQueryMsg, AppMsg>;
pub type InstantiateMsg<AppMsg = Empty> = MiddlewareInstantiateMsg<BaseInstantiateMsg, AppMsg>;
pub type MigrateMsg<AppMsg = Empty> = MiddlewareMigrateMsg<BaseMigrateMsg, AppMsg>;

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
#[allow(unused)]
use cw_controllers::AdminResponse;

/// Used by Module Factory to instantiate App
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    pub ans_host_address: String,
}

#[cosmwasm_schema::cw_serde]
pub enum BaseExecuteMsg {
    /// Updates the base config
    UpdateConfig { ans_host_address: Option<String> },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum BaseQueryMsg {
    /// Returns [`AppConfigResponse`]
    #[returns(AppConfigResponse)]
    Config {},
    /// Returns the admin.
    #[returns(AdminResponse)]
    Admin {},
}

#[cosmwasm_schema::cw_serde]
pub struct AppConfigResponse {
    pub proxy_address: Addr,
    pub ans_host_address: Addr,
    pub manager_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct BaseMigrateMsg {}
