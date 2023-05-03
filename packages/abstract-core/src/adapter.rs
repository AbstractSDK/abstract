//! # Abstract Api Base
//!
//! `abstract_core::adapter` implements shared functionality that's useful for creating new Abstract adapters.
//!
//! ## Description
//! An Abstract adapter contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the Account.
//! The adapter structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use crate::base::{
    ExecuteMsg as MiddlewareExecMsg, InstantiateMsg as MiddlewareInstantiateMsg,
    QueryMsg as MiddlewareQueryMsg,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
use serde::Serialize;

pub type ExecuteMsg<Request = Empty, ReceiveMsg = Empty> =
    MiddlewareExecMsg<BaseExecuteMsg, AdapterRequestMsg<Request>, ReceiveMsg>;
pub type QueryMsg<ModuleMsg = Empty> = MiddlewareQueryMsg<BaseQueryMsg, ModuleMsg>;
pub type InstantiateMsg<ModuleMsg = Empty> =
    MiddlewareInstantiateMsg<BaseInstantiateMsg, ModuleMsg>;

/// Trait indicates that the type is used as an app message
/// in the [`ExecuteMsg`] enum.
/// Enables [`Into<ExecuteMsg>`] for BOOT fn-generation support.
pub trait AdapterExecuteMsg: Serialize {}

impl<T: AdapterExecuteMsg, R: Serialize> From<T> for ExecuteMsg<T, R> {
    fn from(adapter_msg: T) -> Self {
        Self::Module(AdapterRequestMsg {
            proxy_address: None,
            request: adapter_msg,
        })
    }
}

impl AdapterExecuteMsg for Empty {}

/// Trait indicates that the type is used as an api message
/// in the [`QueryMsg`] enum.
/// Enables [`Into<QueryMsg>`] for BOOT fn-generation support.
pub trait AdapterQueryMsg: Serialize {}

impl<T: AdapterQueryMsg> From<T> for QueryMsg<T> {
    fn from(app: T) -> Self {
        Self::Module(app)
    }
}

impl AdapterQueryMsg for Empty {}

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::ProposeModules`].
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    /// Used to easily perform address translation
    pub ans_host_address: String,
    /// Used to verify senders
    pub version_control_address: String,
}

impl<RequestMsg, ReceiveMsg> From<BaseExecuteMsg>
    for MiddlewareExecMsg<BaseExecuteMsg, RequestMsg, ReceiveMsg>
{
    fn from(adapter_msg: BaseExecuteMsg) -> Self {
        Self::Base(adapter_msg)
    }
}

impl<RequestMsg, Request, BaseExecMsg> From<AdapterRequestMsg<RequestMsg>>
    for MiddlewareExecMsg<BaseExecMsg, AdapterRequestMsg<RequestMsg>, Request>
{
    fn from(request_msg: AdapterRequestMsg<RequestMsg>) -> Self {
        Self::Module(request_msg)
    }
}

/// An adapter request.
/// If proxy is None, then the sender must be an Account manager and the proxy address is extrapolated from the Account id.
#[cosmwasm_schema::cw_serde]
pub struct AdapterRequestMsg<Request> {
    pub proxy_address: Option<String>,
    /// The actual request
    pub request: Request,
}

impl<Request: Serialize> AdapterRequestMsg<Request> {
    pub fn new(proxy_address: Option<String>, request: Request) -> Self {
        Self {
            proxy_address,
            request,
        }
    }
}

/// Configuration message for the adapter
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
#[cfg_attr(feature = "boot", impl_into(ExecuteMsg<T>))]
pub enum BaseExecuteMsg {
    /// Add or remove authorized addresses
    /// If an authorized address is both in to_add and to_remove, it will be removed.
    UpdateAuthorizedAddresses {
        to_add: Vec<String>,
        to_remove: Vec<String>,
    },
    /// Remove the adapter
    Remove {},
}

/// Query adapter message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
#[cfg_attr(feature = "boot", impl_into(QueryMsg < ModuleMsg >))]
pub enum BaseQueryMsg {
    /// Returns [`AdapterConfigResponse`].
    #[returns(AdapterConfigResponse)]
    Config {},
    /// Returns [`AuthorizedAddressesResponse`].
    #[returns(AuthorizedAddressesResponse)]
    AuthorizedAddresses { proxy_address: String },
}

impl<T> From<BaseQueryMsg> for QueryMsg<T> {
    fn from(base: BaseQueryMsg) -> Self {
        Self::Base(base)
    }
}

#[cosmwasm_schema::cw_serde]
pub struct AdapterConfigResponse {
    pub version_control_address: Addr,
    pub ans_host_address: Addr,
    pub dependencies: Vec<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct AuthorizedAddressesResponse {
    /// Contains all authorized addresses
    pub addresses: Vec<Addr>,
}
