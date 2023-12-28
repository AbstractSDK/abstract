use abstract_core::AbstractError;
use abstract_interface::AbstractInterfaceError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbstractClientError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    Interface(#[from] AbstractInterfaceError),

    #[error("{0}")]
    CwOrch(#[from] cw_orch::prelude::CwOrchError),

    #[error("{0}")]
    Semver(#[from] semver::Error),

    #[error("Module not installed")]
    ModuleNotInstalled {},
}
