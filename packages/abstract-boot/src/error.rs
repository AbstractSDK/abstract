use abstract_core::AbstractError;
use boot_core::BootError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AbstractBootError {
    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Boot(#[from] BootError),

    #[error("JSON Conversion Error")]
    SerdeJson(#[from] ::serde_json::Error),

    #[error("{0}")]
    Std(#[from] StdError),
}

impl AbstractBootError {
    pub fn root(&self) -> &dyn std::error::Error {
        match self {
            AbstractBootError::Boot(e) => e.root(),
            _ => panic!("Unexpected error type"),
        }
    }
}
