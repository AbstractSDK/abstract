//! # Abstract App
//!
//! `abstract_std::app` implements shared functionality that's useful for creating new Abstract apps.
//!
//! ## Description
//! An app is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract while also being migratable.
use crate::{
    base::{
        ExecuteMsg as EndpointExecMsg, InstantiateMsg as EndpointInstantiateMsg,
        MigrateMsg as EndpointMigrateMsg, QueryMsg as EndpointQueryMsg,
    },
    objects::{
        ans_host::AnsHost, module_version::ModuleDataResponse, nested_admin::TopLevelOwnerResponse,
        version_control::VersionControlContract,
    },
    version_control::AccountBase,
};

pub type ExecuteMsg<ModuleMsg = Empty, ReceiveMsg = Empty> =
    EndpointExecMsg<BaseExecuteMsg, ModuleMsg, ReceiveMsg>;
pub type QueryMsg<ModuleMsg = Empty> = EndpointQueryMsg<BaseQueryMsg, ModuleMsg>;
pub type InstantiateMsg<ModuleMsg = Empty> = EndpointInstantiateMsg<BaseInstantiateMsg, ModuleMsg>;
pub type MigrateMsg<ModuleMsg = Empty> = EndpointMigrateMsg<BaseMigrateMsg, ModuleMsg>;

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
#[allow(unused_imports)]
use cw_controllers::AdminResponse;
use serde::Serialize;

/// Trait indicates that the type is used as an app message
/// in the [`ExecuteMsg`] enum.
/// Enables [`Into<ExecuteMsg>`] for BOOT fn-generation support.
pub trait AppExecuteMsg: Serialize {}
impl<T: AppExecuteMsg, R: Serialize> From<T> for ExecuteMsg<T, R> {
    fn from(app: T) -> Self {
        Self::Module(app)
    }
}

impl AppExecuteMsg for Empty {}

/// Trait indicates that the type is used as an app message
/// in the [`QueryMsg`] enum.
/// Enables [`Into<QueryMsg>`] for BOOT fn-generation support.
pub trait AppQueryMsg: Serialize {}
impl<T: AppQueryMsg> From<T> for QueryMsg<T> {
    fn from(app: T) -> Self {
        Self::Module(app)
    }
}
impl AppQueryMsg for Empty {}

/// Used by Module Factory to instantiate App
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    pub ans_host_address: String,
    pub version_control_address: String,
    pub account_base: AccountBase,
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg<T>))]
pub enum BaseExecuteMsg {
    /// Updates the base config
    UpdateConfig {
        ans_host_address: Option<String>,
        version_control_address: Option<String>,
    },
}

impl<T> From<BaseExecuteMsg> for ExecuteMsg<T> {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::Base(base)
    }
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg<ModuleMsg>))]
pub enum BaseQueryMsg {
    /// Returns [`AppConfigResponse`]
    #[returns(AppConfigResponse)]
    BaseConfig {},
    /// Returns the admin.
    /// Returns [`AdminResponse`]
    #[returns(AdminResponse)]
    BaseAdmin {},
    /// Returns module data
    /// Returns [`ModuleDataResponse`]
    #[returns(ModuleDataResponse)]
    ModuleData {},
    /// Returns top level owner
    /// Returns [`TopLevelOwnerResponse`]
    #[returns(TopLevelOwnerResponse)]
    TopLevelOwner {},
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

/// The BaseState contains the main addresses needed for sending and verifying messages
#[cosmwasm_schema::cw_serde]
pub struct AppState {
    /// Proxy contract address for relaying transactions
    pub proxy_address: Addr,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
    /// Used to verify requests
    pub version_control: VersionControlContract,
}
