use cosmwasm_std::Addr;
use cw_asset::AssetError;
use os::{objects::AssetEntry, AbstractOsError};
use std::fmt::{Display, Formatter};
use thiserror::Error;

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

#[derive(Error, Debug, PartialEq)]
pub enum AbstractSdkError {
    #[error("Abstract OS error in the sdk: {0}")]
    AbstractOs(#[from] AbstractOsError),

    #[error("Std error encountered in sdk: {0}")]
    Std(#[from] cosmwasm_std::StdError),

    #[error("Asset error encountered in sdk while handling assets: {0}")]
    Asset(#[from] AssetError),

    // #[error("cw math overflow error: {0}")]
    // Overflow(#[from] OverflowError),

    // #[error("Semver error encountered while handling os object: {0}")]
    // Semver(#[from] SemverError),

    // #[error("Semver error encountered while handling os object: {0}")]
    // CwSemver(#[from] CwSemverError),
    #[error("Missing handler for {endpoint}")]
    MissingHandler { endpoint: String },

    // missing module error
    #[error("Missing module {module}")]
    MissingModule { module: String },

    #[error("Module {module} is not a dependency of this contract.")]
    MissingDependency { module: String },

    // missing asset error
    #[error("Asset {asset} is not registered on your OS. Please register it first.")]
    MissingAsset { asset: AssetEntry },

    // caller not Manager error
    #[error("Address {0} is not the Manager of OS {1}.")]
    NotManager(Addr, u32),

    // caller not Proxy error
    #[error("Address {0} is not the Proxy of OS {1}.")]
    NotProxy(Addr, u32),

    // unknown OS id error
    #[error("Unknown OS id {os_id} on version control {version_control_addr}. Please ensure that you are using the correct OS id and version control address.")]
    UnknownOsId {
        os_id: u32,
        version_control_addr: Addr,
    },

    // failed to query os id
    #[error("Failed to query OS id on contract {contract_addr}. Please ensure that the contract is a Manager or Proxy contract.")]
    FailedToQueryOsId { contract_addr: Addr },

    // module not found in version registry
    #[error("Module {module} not found in version registry {registry_addr}.")]
    ModuleNotFound { module: String, registry_addr: Addr },

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
