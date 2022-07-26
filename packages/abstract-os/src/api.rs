//! # Abstract API Base
//!
//! `abstract_os::api` implements shared functionality that's useful for creating new Abstract APIs.
//!
//! ## Description
//! An Abstract API contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the OS.
//! The API structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use cosmwasm_std::{Addr, Empty};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::AddApi`].
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiInstantiateMsg {
    /// Used to easily perform address translation
    pub memory_address: String,
    /// Used to verify senders
    pub version_control_address: String,
}

/// Interface to the API.
/// Equivalent to ExecuteMsg
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ApiInterfaceMsg<T: Serialize = Empty> {
    /// An API request. Forwards the msg to the associated proxy.
    Request(ApiRequestMsg<T>),
    /// A configuration message to whitelist traders.
    Configure(ApiExecuteMsg),
}

impl<T: Serialize> From<ApiExecuteMsg> for ApiInterfaceMsg<T> {
    fn from(api_msg: ApiExecuteMsg) -> Self {
        Self::Configure(api_msg)
    }
}

impl<T: Serialize> From<ApiRequestMsg<T>> for ApiInterfaceMsg<T> {
    fn from(request_msg: ApiRequestMsg<T>) -> Self {
        Self::Request(request_msg)
    }
}
/// An API request.
/// The api contract forwards the generated msg to the optionally attached proxy addr.
/// If proxy is None, then the sender must be an OS manager and the proxy address is extrapolated from the OS id.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiRequestMsg<T: Serialize = Empty> {
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
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ApiExecuteMsg {
    /// Add or remove traders
    /// If a trader is both in to_add and to_remove, it will be removed.
    UpdateTraders {
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    },
    Remove {},
}

/// Query API message
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiQueryMsg {
    /// Returns [`QueryApiConfigResponse`].
    Config {},
    /// Returns [`QueryTradersResponse`].
    /// TODO: enable pagination of some sort
    Traders { proxy_address: String },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueryApiConfigResponse {
    pub version_control_address: Addr,
    pub memory_address: Addr,
    pub dependencies: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct QueryTradersResponse {
    /// Contains all traders
    pub traders: Vec<Addr>,
}
