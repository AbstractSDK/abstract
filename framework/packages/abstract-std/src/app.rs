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
    objects::{gov_type::TopLevelOwnerResponse, module_version::ModuleDataResponse},
    version_control::Account,
};

pub type ExecuteMsg<ModuleMsg = Empty> = EndpointExecMsg<BaseExecuteMsg, ModuleMsg>;
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
impl<T: AppExecuteMsg> From<T> for ExecuteMsg<T> {
    fn from(module: T) -> Self {
        Self::Module(module)
    }
}

impl AppExecuteMsg for Empty {}

/// Trait indicates that the type is used as an app message
/// in the [`QueryMsg`] enum.
/// Enables [`Into<QueryMsg>`] for BOOT fn-generation support.
pub trait AppQueryMsg: Serialize {}
impl<T: AppQueryMsg> From<T> for QueryMsg<T> {
    fn from(module: T) -> Self {
        Self::Module(module)
    }
}
impl AppQueryMsg for Empty {}

/// Used by Module Factory to instantiate App
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    pub account: Account,
}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum BaseExecuteMsg {
    /// Updates the base config
    UpdateConfig {},
}

impl<T> From<BaseExecuteMsg> for ExecuteMsg<T> {
    fn from(base: BaseExecuteMsg) -> Self {
        Self::Base(base)
    }
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
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
    pub account: Addr,
    pub ans_host_address: Addr,
    pub version_control_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct BaseMigrateMsg {}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[cosmwasm_schema::cw_serde]
pub struct AppState {
    /// Account contract address for proxying transactions
    pub account: Account,
}
