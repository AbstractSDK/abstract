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

pub type ExecuteMsg<T, R = Empty> = MiddlewareExecMsg<BaseExecuteMsg, T, R>;
pub type QueryMsg<T = Empty> = MiddlewareQueryMsg<BaseQueryMsg, T>;
pub type InstantiateMsg<T = Empty> = MiddlewareInstantiateMsg<BaseInstantiateMsg, T>;
pub type MigrateMsg<T = Empty> = MiddlewareMigrateMsg<BaseMigrateMsg, T>;

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
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
