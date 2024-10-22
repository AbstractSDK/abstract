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

    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Instantiate2(#[from] cosmwasm_std::Instantiate2AddressError),

    #[error("Abstract is not deployed on this chain")]
    NotDeployed {},

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

    pub fn downcast<E>(self) -> cw_orch::anyhow::Result<E>
    where
        E: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        match self {
            AbstractInterfaceError::Orch(e) => e.downcast(),
            _ => panic!("Unexpected error type"),
        }
    }
}
