use abstract_std::{objects::dependency::StaticDependency, AbstractError};
use cosmwasm_std::StdError;
use cw_orch::prelude::CwOrchError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbstractInterfaceError {
    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Orch(#[from] CwOrchError),

    #[cfg(feature = "interchain")]
    #[error(transparent)]
    OrchInterchain(#[from] cw_orch_interchain::core::InterchainError),

    #[error("JSON Conversion Error: {0}")]
    SerdeJson(#[from] ::serde_json::Error),

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Instantiate2(#[from] cosmwasm_std::Instantiate2AddressError),

    #[cfg(feature = "daemon")]
    #[error(transparent)]
    Daemon(#[from] cw_orch::daemon::DaemonError),

    #[error("Abstract is not deployed on this chain")]
    NotDeployed {},

    #[error("Module Not Found {0}")]
    ModuleNotFound(String),

    #[error("No need to update {0}")]
    NotUpdated(String),

    #[error(transparent)]
    Semver(#[from] semver::Error),

    #[error("No matching module deployed {0:?}")]
    NoMatchingModule(StaticDependency),
}

impl AbstractInterfaceError {
    pub fn root(&self) -> &dyn std::error::Error {
        match self {
            AbstractInterfaceError::Orch(e) => e.root(),
            _ => panic!("Unexpected error type"),
        }
    }
}
