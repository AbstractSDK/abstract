//! # Abstract Api Base
//!
//! `abstract_core::adapter` implements shared functionality that's useful for creating new Abstract adapters.
//!
//! ## Description
//! An Abstract adapter contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the Account.
//! The adapter structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use crate::{
    base::{
        ExecuteMsg as MiddlewareExecMsg, InstantiateMsg as MiddlewareInstantiateMsg,
        QueryMsg as MiddlewareQueryMsg,
    },
    objects::module_version::ModuleDataResponse,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
use schemars::JsonSchema;
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
#[derive(Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct AdapterRequestMsg<Request> {
    pub proxy_address: Option<String>,
    /// The actual request
    #[serde(flatten)]
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

// serde attributes remain it compatible with previous versions in cases where proxy_address is omitted
#[derive(Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(crate = "::cosmwasm_schema::serde")]
#[schemars(crate = "::cosmwasm_schema::schemars")]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct BaseExecuteMsg {
    /// The Proxy address for which to apply the configuration
    /// If None, the sender must be an Account manager and the configuration is applied to its associated proxy.
    /// If Some, the sender must be a direct or indirect owner (through sub-accounts) of the specified proxy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_address: Option<String>,
    // The actual base message
    #[serde(flatten)]
    pub msg: AdapterBaseMsg,
}

/// Configuration message for the adapter
#[cosmwasm_schema::cw_serde]
pub enum AdapterBaseMsg {
    /// Add or remove authorized addresses
    /// If an authorized address is both in to_add and to_remove, it will be removed.
    UpdateAuthorizedAddresses {
        to_add: Vec<String>,
        to_remove: Vec<String>,
    },
}

/// Query adapter message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg<ModuleMsg>))]
pub enum BaseQueryMsg {
    /// Returns [`AdapterConfigResponse`].
    #[returns(AdapterConfigResponse)]
    BaseConfig {},
    /// Returns [`AuthorizedAddressesResponse`].
    #[returns(AuthorizedAddressesResponse)]
    AuthorizedAddresses { proxy_address: String },
    /// Returns module data
    #[returns(ModuleDataResponse)]
    ModuleData {},
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

#[cfg(test)]
mod tests {
    use cosmwasm_std::to_json_binary;

    use super::*;

    type AdapterExecuteMsg = ExecuteMsg<Empty>;
    type PreviousAdapterExecuteMsg = MiddlewareExecMsg<AdapterBaseMsg, Empty>;

    #[test]
    fn compatible_msg() {
        let msg = to_json_binary(&AdapterExecuteMsg::Base(BaseExecuteMsg {
            proxy_address: None,
            msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                to_add: vec![String::from("abc")],
                to_remove: vec![],
            },
        }))
        .unwrap();

        let previous_msg = to_json_binary(&PreviousAdapterExecuteMsg::Base(
            AdapterBaseMsg::UpdateAuthorizedAddresses {
                to_add: vec![String::from("abc")],
                to_remove: vec![],
            },
        ))
        .unwrap();

        assert_eq!(msg, previous_msg);
    }
}
