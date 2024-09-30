//! # Abstract Api Base
//!
//! `abstract_std::adapter` implements shared functionality that's useful for creating new Abstract adapters.
//!
//! ## Description
//! An Abstract adapter contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the Account.
//! The adapter structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
use serde::Serialize;

use crate::{
    base::{
        ExecuteMsg as MiddlewareExecMsg, InstantiateMsg as MiddlewareInstantiateMsg,
        QueryMsg as MiddlewareQueryMsg,
    },
    objects::{ans_host::AnsHost, module_version::ModuleDataResponse, registry::RegistryContract},
};

pub type ExecuteMsg<Request = Empty> =
    MiddlewareExecMsg<BaseExecuteMsg, AdapterRequestMsg<Request>>;
pub type QueryMsg<ModuleMsg = Empty> = MiddlewareQueryMsg<BaseQueryMsg, ModuleMsg>;
pub type InstantiateMsg<ModuleMsg = Empty> =
    MiddlewareInstantiateMsg<BaseInstantiateMsg, ModuleMsg>;

/// Trait indicates that the type is used as an app message
/// in the [`ExecuteMsg`] enum.
/// Enables [`Into<ExecuteMsg>`] for BOOT fn-generation support.
pub trait AdapterExecuteMsg: Serialize {}
impl<T: AdapterExecuteMsg> From<T> for ExecuteMsg<T> {
    fn from(request: T) -> Self {
        Self::Module(AdapterRequestMsg {
            account_address: None,
            request,
        })
    }
}

impl AdapterExecuteMsg for Empty {}

/// Trait indicates that the type is used as an api message
/// in the [`QueryMsg`] enum.
/// Enables [`Into<QueryMsg>`] for BOOT fn-generation support.
pub trait AdapterQueryMsg: Serialize {}

impl<T: AdapterQueryMsg> From<T> for QueryMsg<T> {
    fn from(module: T) -> Self {
        Self::Module(module)
    }
}

impl AdapterQueryMsg for Empty {}

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::registry::ExecuteMsg::ProposeModules`].
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    /// Used to easily perform address translation
    pub ans_host_address: String,
    /// Used to verify senders
    pub version_control_address: String,
}

impl<RequestMsg> From<BaseExecuteMsg> for MiddlewareExecMsg<BaseExecuteMsg, RequestMsg> {
    fn from(adapter_msg: BaseExecuteMsg) -> Self {
        Self::Base(adapter_msg)
    }
}

impl<RequestMsg, BaseExecMsg> From<AdapterRequestMsg<RequestMsg>>
    for MiddlewareExecMsg<BaseExecMsg, AdapterRequestMsg<RequestMsg>>
{
    fn from(request_msg: AdapterRequestMsg<RequestMsg>) -> Self {
        Self::Module(request_msg)
    }
}

/// An adapter request.
/// If proxy is None, then the sender must be an Account manager and the proxy address is extrapolated from the Account id.
#[cosmwasm_schema::cw_serde]
pub struct AdapterRequestMsg<Request> {
    pub account_address: Option<String>,
    /// The actual request
    pub request: Request,
}

impl<Request: Serialize> AdapterRequestMsg<Request> {
    pub fn new(account_address: Option<String>, request: Request) -> Self {
        Self {
            account_address,
            request,
        }
    }
}

// serde attributes remain it compatible with previous versions in cases where proxy_address is omitted
#[cosmwasm_schema::cw_serde]
pub struct BaseExecuteMsg {
    /// The account address for which to apply the configuration
    /// If None, the sender must be an Account
    /// If Some, the sender must be a direct or indirect owner (through sub-accounts) of the specified account.
    pub account_address: Option<String>,
    // The actual base message
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
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum BaseQueryMsg {
    /// Returns [`AdapterConfigResponse`].
    #[returns(AdapterConfigResponse)]
    BaseConfig {},
    /// Returns [`AuthorizedAddressesResponse`].
    #[returns(AuthorizedAddressesResponse)]
    AuthorizedAddresses { proxy_address: String },
    /// Returns module data
    /// Returns [`ModuleDataResponse`].
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

/// The BaseState contains the main addresses needed for sending and verifying messages
/// Every DApp should use the provided **ans_host** contract for token/contract address resolution.
#[cosmwasm_schema::cw_serde]
pub struct AdapterState {
    /// Used to verify requests
    pub version_control: RegistryContract,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
}
