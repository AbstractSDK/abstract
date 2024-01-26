//! # Represents Abstract Client Errors

use abstract_core::{objects::validation::ValidationError, AbstractError};
use abstract_interface::AbstractInterfaceError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
/// Error type for the abstract client crate.
#[allow(missing_docs)] // Error type names should be self-explanatory
pub enum AbstractClientError {
    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    CosmwasmStd(#[from] StdError),

    #[error("{0}")]
    Interface(#[from] AbstractInterfaceError),

    #[error("{0}")]
    CwOrch(#[from] cw_orch::prelude::CwOrchError),

    #[error("{0}")]
    Semver(#[from] semver::Error),

    #[error("{0}")]
    ValidationError(#[from] ValidationError),

    #[error("Module not installed")]
    ModuleNotInstalled {},

    #[error("Account is Renounced and does not have an owner.")]
    RenouncedAccount {},
}
