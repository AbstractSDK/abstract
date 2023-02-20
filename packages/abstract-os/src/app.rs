//! # Abstract App
//!
//! `abstract_os::app` implements shared functionality that's useful for creating new Abstract apps.
//!
//! ## Description
//! An app is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract while also being migratable.
use crate::base::{
    ExecuteMsg as EndpointExecMsg, InstantiateMsg as EndpointInstantiateMsg,
    MigrateMsg as EndpointMigrateMsg, QueryMsg as EndpointQueryMsg,
};

pub type ExecuteMsg<AppMsg, ReceiveMsg = Empty> =
    EndpointExecMsg<BaseExecuteMsg, AppMsg, ReceiveMsg>;
pub type QueryMsg<AppMsg = Empty> = EndpointQueryMsg<BaseQueryMsg, AppMsg>;
pub type InstantiateMsg<AppMsg = Empty> = EndpointInstantiateMsg<BaseInstantiateMsg, AppMsg>;
pub type MigrateMsg<AppMsg = Empty> = EndpointMigrateMsg<BaseMigrateMsg, AppMsg>;

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
use cw_controllers::AdminResponse;
use serde::Serialize;

/// Trait indicates that the type is used as an app message
/// in the [`ExecuteMsg`] enum.
/// Enables [`Into<ExecuteMsg>`] for BOOT fn-generation support.
pub trait AppExecuteMsg: Serialize {}
impl<T: AppExecuteMsg, R: Serialize> From<T> for ExecuteMsg<T, R> {
    fn from(app: T) -> Self {
        Self::App(app)
    }
}

/// Trait indicates that the type is used as an app message
/// in the [`QueryMsg`] enum.
/// Enables [`Into<QueryMsg>`] for BOOT fn-generation support.
pub trait AppQueryMsg: Serialize {}
impl<T: AppQueryMsg> From<T> for QueryMsg<T> {
    fn from(app: T) -> Self {
        Self::App(app)
    }
}
impl AppQueryMsg for Empty {}

/// Used by Module Factory to instantiate App
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    pub ans_host_address: String,
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
#[cfg_attr(feature = "boot", impl_into(ExecuteMsg<T>))]
pub enum BaseExecuteMsg {
    /// Updates the base config
    UpdateConfig { ans_host_address: Option<String> },
}

impl<T> From<BaseExecuteMsg> for ExecuteMsg<T> {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::Base(base)
    }
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
#[cfg_attr(feature = "boot", impl_into(QueryMsg<AppMsg>))]
pub enum BaseQueryMsg {
    /// Returns [`AppConfigResponse`]
    #[returns(AppConfigResponse)]
    Config {},
    /// Returns the admin.
    #[returns(AdminResponse)]
    Admin {},
}

impl<T> From<BaseQueryMsg> for QueryMsg<T> {
    fn from(base: BaseQueryMsg) -> Self {
        Self::Base(base)
    }
}

#[cosmwasm_schema::cw_serde]
pub struct AppConfigResponse {
    pub proxy_address: Addr,
    pub ans_host_address: Addr,
    pub manager_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct BaseMigrateMsg {}
