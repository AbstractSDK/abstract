//! # Abstract App
//!
//! `abstract_core::app` implements shared functionality that's useful for creating new Abstract apps.
//!
//! ## Description
//! An app is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract while also being migratable.
use crate::base::{
    ExecuteMsg as EndpointExecMsg, InstantiateMsg as EndpointInstantiateMsg,
    MigrateMsg as EndpointMigrateMsg, QueryMsg as EndpointQueryMsg,
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

/// Enable [`cw_orch_cli::ParseCwMsg`] for Module Execute messages that implement that trait
#[cfg(feature = "interface")]
impl<T: AppExecuteMsg + cw_orch_cli::ParseCwMsg, R: Serialize> cw_orch_cli::ParseCwMsg
    for ExecuteMsg<T, R>
{
    fn cw_parse(
        state_interface: &impl cw_orch::state::StateInterface,
    ) -> cw_orch::anyhow::Result<Self> {
        Ok(Self::Module(T::cw_parse(state_interface)?))
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

/// Enable [`cw_orch_cli::ParseCwMsg`] for Module Query messages that implement that trait
#[cfg(feature = "interface")]
impl<T: AppQueryMsg + cw_orch_cli::ParseCwMsg> cw_orch_cli::ParseCwMsg for QueryMsg<T> {
    fn cw_parse(
        state_interface: &impl cw_orch::state::StateInterface,
    ) -> cw_orch::anyhow::Result<Self> {
        Ok(Self::Module(T::cw_parse(state_interface)?))
    }
}

impl AppQueryMsg for Empty {}

/// Enable [`cw_orch_cli::ParseCwMsg`] for Module Instantiate messages that implement that trait
#[cfg(feature = "interface")]
impl<T: cw_orch_cli::ParseCwMsg> cw_orch_cli::ParseCwMsg for InstantiateMsg<T> {
    fn cw_parse(
        state_interface: &impl cw_orch::state::StateInterface,
    ) -> cw_orch::anyhow::Result<Self> {
        Ok(Self {
            base: BaseInstantiateMsg {
                ans_host_address: state_interface.get_address(crate::ANS_HOST)?.into_string(),
            },
            module: T::cw_parse(state_interface)?,
        })
    }
}

/// Enable [`cw_orch_cli::ParseCwMsg`] for Module Migrate messages that implement that trait
#[cfg(feature = "interface")]
impl<T: cw_orch_cli::ParseCwMsg> cw_orch_cli::ParseCwMsg for MigrateMsg<T> {
    fn cw_parse(
        state_interface: &impl cw_orch::state::StateInterface,
    ) -> cw_orch::anyhow::Result<Self> {
        Ok(Self {
            base: BaseMigrateMsg {},
            module: T::cw_parse(state_interface)?,
        })
    }
}

/// Used by Module Factory to instantiate App
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    pub ans_host_address: String,
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg<T>))]
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
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg<ModuleMsg>))]
pub enum BaseQueryMsg {
    /// Returns [`AppConfigResponse`]
    #[returns(AppConfigResponse)]
    BaseConfig {},
    /// Returns the admin.
    #[returns(AdminResponse)]
    BaseAdmin {},
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
