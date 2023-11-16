#![allow(missing_docs)]
use crate::core::{objects::AssetEntry, AbstractError};
use abstract_core::objects::AccountId;
use cosmwasm_std::Addr;
use cw_asset::AssetError;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// Error type for the abstract module endpoints.
#[derive(Error, Debug, PartialEq)]
pub struct EndpointError {
    #[source]
    source: AbstractSdkError,
    module_id: String,
}

impl Display for EndpointError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error in {} - {}", self.module_id, self.source)
    }
}
/// Error type for the abstract sdk crate.
#[derive(Error, Debug, PartialEq)]
pub enum AbstractSdkError {
    #[error("Abstract Account error in the sdk: {0}")]
    Abstract(#[from] AbstractError),

    #[error("Std error encountered in sdk: {0}")]
    Std(#[from] cosmwasm_std::StdError),

    #[error("Asset error encountered in sdk while handling assets: {0}")]
    Asset(#[from] AssetError),

    // #[error("cw math overflow error: {0}")]
    // Overflow(#[from] OverflowError),

    // #[error("Semver error encountered while handling account object: {0}")]
    // Semver(#[from] SemverError),

    // #[error("Semver error encountered while handling account object: {0}")]
    // CwSemver(#[from] CwSemverError),
    #[error("Missing handler for {endpoint}")]
    MissingHandler { endpoint: String },

    // missing module error
    #[error("Missing module {module}")]
    MissingModule { module: String },

    #[error("Module {module} is not a dependency of this contract.")]
    MissingDependency { module: String },

    // missing asset error
    #[error("Asset {asset} is not registered on your Account. Please register it first.")]
    MissingAsset { asset: AssetEntry },

    // caller not Manager error
    #[error("Address {0} is not the Manager of Account {1}.")]
    NotManager(Addr, AccountId),

    // caller not Proxy error
    #[error("Address {0} is not the Proxy of Account {1}.")]
    NotProxy(Addr, AccountId),

    // unknown Account id error
    #[error("Unknown Account id {account_id} on version control {version_control_addr}. Please ensure that you are using the correct Account id and version control address.")]
    UnknownAccountId {
        account_id: AccountId,
        version_control_addr: Addr,
    },

    // failed to query account id
    #[error("Failed to query Account id on contract {contract_addr}. Please ensure that the contract is a Manager or Proxy contract.")]
    FailedToQueryAccountId { contract_addr: Addr },

    // module not found in version registry
    #[error("Module {module} not found in version registry {registry_addr}.")]
    ModuleNotFound { module: String, registry_addr: Addr },

    // module not found in version registry
    #[error("Standalone {code_id} not found in version registry {registry_addr}.")]
    StandaloneNotFound { code_id: u64, registry_addr: Addr },

    // callback not called by IBC client
    #[error("IBC callback called by {caller} instead of IBC client {client_addr}.")]
    CallbackNotCalledByIbcClient {
        caller: Addr,
        client_addr: Addr,
        module: String,
    },

    // admin of proxy is not set
    #[error("Admin of proxy {proxy_addr} is not set.")]
    AdminNotSet { proxy_addr: Addr },
}

impl AbstractSdkError {
    pub fn generic_err(msg: impl Into<String>) -> Self {
        AbstractSdkError::Std(cosmwasm_std::StdError::generic_err(msg))
    }
}
