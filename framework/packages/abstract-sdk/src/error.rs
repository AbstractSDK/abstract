#![allow(missing_docs)]
use std::fmt::{Display, Formatter};

use cosmwasm_std::Addr;
use cw_asset::AssetError;
use thiserror::Error;

use crate::std::{objects::AssetEntry, AbstractError};

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

    // callback not called by IBC client
    #[error("IBC callback called by {caller} instead of IBC client {client_addr}.")]
    CallbackNotCalledByIbcClient {
        caller: Addr,
        client_addr: Addr,
        module: String,
    },

    // callback not called by IBC host
    #[error("Module {module} Ibc Endpoint called by {caller} instead of IBC host {host_addr}.")]
    ModuleIbcNotCalledByHost {
        caller: Addr,
        host_addr: Addr,
        module: String,
    },

    // callback not called by IBC host
    #[error("Called an IBC module action on {0}, when no endpoint was registered.")]
    NoModuleIbcHandler(String),

    // admin of proxy is not set
    #[error("Admin of proxy {proxy_addr} is not set.")]
    AdminNotSet { proxy_addr: Addr },

    // Query from api object failed
    #[error("API query for {api} failed in {module_id}: {error}")]
    ApiQuery {
        api: String,
        module_id: String,
        error: Box<AbstractError>,
    },

    // Queried address is not a module
    #[error("Queried address {addr} is not a module: {err}")]
    NotAModule { addr: Addr, err: String },

    // Queried address is not a module
    #[error(
        "Queried address {addr} is a module ({module}) but has the wrong stored address : {err}"
    )]
    WrongModuleInfo {
        addr: Addr,
        module: String,
        err: String,
    },

    // The caller can only execute admin operations on one module
    #[error(
        "The action triggered can only execute admin operations on {admin_for} and not {current_contract}"
    )]
    NotAdminFor {
        admin_for: Addr,
        current_contract: Addr,
    },

    // This call needs to be an admin call
    #[error("This action can only be triggered via an Account Admin call")]
    OnlyAdmin {},
}

impl AbstractSdkError {
    pub fn generic_err(msg: impl Into<String>) -> Self {
        AbstractSdkError::Std(cosmwasm_std::StdError::generic_err(msg))
    }
}
