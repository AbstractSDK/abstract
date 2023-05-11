use abstract_core::AbstractError;
use cosmwasm_std::StdError;
use cw_orch::prelude::CwOrchError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbstractInterfaceError {
    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Orch(#[from] CwOrchError),

    #[error("JSON Conversion Error")]
    SerdeJson(#[from] ::serde_json::Error),

    #[error("{0}")]
    Std(#[from] StdError),
}

impl AbstractInterfaceError {
    pub fn root(&self) -> &dyn std::error::Error {
        match self {
            AbstractInterfaceError::Orch(e) => e.root(),
            _ => panic!("Unexpected error type"),
        }
    }
}
