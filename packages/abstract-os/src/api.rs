//! # Abstract API Base
//!
//! `abstract_os::api` implements shared functionality that's useful for creating new Abstract APIs.
//!
//! ## Description
//! An Abstract API contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the OS.
//! The API structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
use serde::Serialize;

use crate::middleware::{
    ExecuteMsg as MiddlewareExecMsg, InstantiateMsg as MiddlewareInstantiateMsg,
    QueryMsg as MiddlewareQueryMsg,
};

pub type ExecuteMsg<T, R = Empty> = MiddlewareExecMsg<BaseExecuteMsg, ApiRequestMsg<T>, R>;
pub type QueryMsg<T = Empty> = MiddlewareQueryMsg<BaseQueryMsg, T>;
pub type InstantiateMsg<T = Empty> = MiddlewareInstantiateMsg<BaseInstantiateMsg, T>;

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::AddApi`].
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    /// Used to easily perform address translation
    pub memory_address: String,
    /// Used to verify senders
    pub version_control_address: String,
}

impl<T, R> From<BaseExecuteMsg> for MiddlewareExecMsg<BaseExecuteMsg, T, R> {
    fn from(api_msg: BaseExecuteMsg) -> Self {
        Self::Base(api_msg)
    }
}

impl<T, R, Q> From<ApiRequestMsg<T>> for MiddlewareExecMsg<Q, ApiRequestMsg<T>, R> {
    fn from(request_msg: ApiRequestMsg<T>) -> Self {
        Self::App(request_msg)
    }
}

/// An API request.
/// If proxy is None, then the sender must be an OS manager and the proxy address is extrapolated from the OS id.
#[cosmwasm_schema::cw_serde]
pub struct ApiRequestMsg<T> {
    pub proxy_address: Option<String>,
    /// The actual request
    pub request: T,
}

impl<T: Serialize> ApiRequestMsg<T> {
    pub fn new(proxy_address: Option<String>, request: T) -> Self {
        Self {
            proxy_address,
            request,
        }
    }
}

/// Configuration message for the API
#[cosmwasm_schema::cw_serde]
pub enum BaseExecuteMsg {
    /// Add or remove traders
    /// If a trader is both in to_add and to_remove, it will be removed.
    UpdateTraders {
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    },
    /// Remove the API
    Remove {},
}

/// Query API message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum BaseQueryMsg {
    /// Returns [`ApiConfigResponse`].
    #[returns(ApiConfigResponse)]
    Config {},
    /// Returns [`TradersResponse`].
    /// TODO: enable pagination
    #[returns(TradersResponse)]
    Traders { proxy_address: String },
}

#[cosmwasm_schema::cw_serde]
pub struct ApiConfigResponse {
    pub version_control_address: Addr,
    pub memory_address: Addr,
    pub dependencies: Vec<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct TradersResponse {
    /// Contains all traders
    pub traders: Vec<Addr>,
}
