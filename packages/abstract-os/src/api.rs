//! # Abstract Api Base
//!
//! `abstract_os::api` implements shared functionality that's useful for creating new Abstract apis.
//!
//! ## Description
//! An Abstract api contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the OS.
//! The api structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use crate::base::{
    ExecuteMsg as MiddlewareExecMsg, InstantiateMsg as MiddlewareInstantiateMsg,
    QueryMsg as MiddlewareQueryMsg,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
use serde::Serialize;

pub type ExecuteMsg<Request, ReceiveMsg = Empty> =
    MiddlewareExecMsg<BaseExecuteMsg, ApiRequestMsg<Request>, ReceiveMsg>;
pub type QueryMsg<AppMsg = Empty> = MiddlewareQueryMsg<BaseQueryMsg, AppMsg>;
pub type InstantiateMsg<AppMsg = Empty> = MiddlewareInstantiateMsg<BaseInstantiateMsg, AppMsg>;

/// Trait indicates that the type is used as an app message
/// in the [`ExecuteMsg`] enum.
/// Enables [`Into<ExecuteMsg>`] for BOOT fn-generation support.
pub trait ApiExecuteMsg: Serialize {}

impl<T: ApiExecuteMsg> From<T> for ExecuteMsg<T> {
    fn from(api_msg: T) -> Self {
        Self::App(ApiRequestMsg {
            proxy_address: None,
            request: api_msg,
        })
    }
}

/// Trait indicates that the type is used as an api message
/// in the [`QueryMsg`] enum.
/// Enables [`Into<QueryMsg>`] for BOOT fn-generation support.
pub trait ApiQueryMsg: Serialize {}

impl<T: ApiQueryMsg> From<T> for QueryMsg<T> {
    fn from(app: T) -> Self {
        Self::App(app)
    }
}

impl ApiQueryMsg for Empty {}

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::AddApi`].
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
    fn from(api_msg: BaseExecuteMsg) -> Self {
        Self::Base(api_msg)
    }
}

impl<RequestMsg, Request, BaseExecMsg> From<ApiRequestMsg<RequestMsg>>
    for MiddlewareExecMsg<BaseExecMsg, ApiRequestMsg<RequestMsg>, Request>
{
    fn from(request_msg: ApiRequestMsg<RequestMsg>) -> Self {
        Self::App(request_msg)
    }
}

/// An api request.
/// If proxy is None, then the sender must be an OS manager and the proxy address is extrapolated from the OS id.
#[cosmwasm_schema::cw_serde]
pub struct ApiRequestMsg<Request> {
    pub proxy_address: Option<String>,
    /// The actual request
    pub request: Request,
}

impl<Request: Serialize> ApiRequestMsg<Request> {
    pub fn new(proxy_address: Option<String>, request: Request) -> Self {
        Self {
            proxy_address,
            request,
        }
    }
}

/// Configuration message for the api
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
#[cfg_attr(feature = "boot", impl_into(ExecuteMsg<T>))]
pub enum BaseExecuteMsg {
    /// Add or remove traders
    /// If a trader is both in to_add and to_remove, it will be removed.
    UpdateTraders {
        to_add: Vec<String>,
        to_remove: Vec<String>,
    },
    /// Remove the api
    Remove {},
}

/// Query api message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
#[cfg_attr(feature = "boot", impl_into(QueryMsg<AppMsg>))]
pub enum BaseQueryMsg {
    /// Returns [`ApiConfigResponse`].
    #[returns(ApiConfigResponse)]
    Config {},
    /// Returns [`TradersResponse`].
    /// TODO: enable pagination
    #[returns(TradersResponse)]
    Traders { proxy_address: String },
}

impl<T> From<BaseQueryMsg> for QueryMsg<T> {
    fn from(base: BaseQueryMsg) -> Self {
        Self::Base(base)
    }
}
#[cosmwasm_schema::cw_serde]
pub struct ApiConfigResponse {
    pub version_control_address: Addr,
    pub ans_host_address: Addr,
    pub dependencies: Vec<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct TradersResponse {
    /// Contains all traders
    pub traders: Vec<Addr>,
}
