//! # Abstract Add-On
//!
//! `abstract_os::add_on` implements shared functionality that's useful for creating new Abstract add-ons.
//!
//! ## Description
//! An add-on is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract while also being migratable.

use crate::middleware::{
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

/// Used by Module Factory to instantiate AddOn
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    pub memory_address: String,
}

#[cosmwasm_schema::cw_serde]
pub enum BaseExecuteMsg {
    /// Updates the base config
    UpdateConfig { memory_address: Option<String> },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum BaseQueryMsg {
    /// Returns [`AddOnConfigResponse`]
    #[returns(AddOnConfigResponse)]
    Config {},
    /// Returns the admin.
    #[returns(AdminResponse)]
    Admin {},
}

#[cosmwasm_schema::cw_serde]
pub struct AddOnConfigResponse {
    pub proxy_address: Addr,
    pub memory_address: Addr,
    pub manager_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct BaseMigrateMsg {}
