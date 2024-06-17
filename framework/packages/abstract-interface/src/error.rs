use abstract_std::AbstractError;
use cosmwasm_std::StdError;
use cw_orch::prelude::CwOrchError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbstractInterfaceError {
    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Orch(#[from] CwOrchError),

    #[cfg(feature="interchain")]
    #[error(transparent)]
    OrchInterchain(#[from] cw_orch_interchain::InterchainError),

    #[error("JSON Conversion Error: {0}")]
    SerdeJson(#[from] ::serde_json::Error),

    #[error("{0}")]
    Std(#[from] StdError),

    #[cfg(feature = "daemon")]
    #[error(transparent)]
    Daemon(#[from] cw_orch::daemon::DaemonError),

    #[error("Abstract is not deployed on this chain")]
    NotDeployed {},
}

impl AbstractInterfaceError {
    pub fn root(&self) -> &dyn std::error::Error {
        match self {
            AbstractInterfaceError::Orch(e) => e.root(),
            _ => panic!("Unexpected error type"),
        }
    }
}
