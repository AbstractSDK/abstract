use abstract_sdk::AbstractSdkError;
use cosmwasm_std::{StdError};
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    // Unauthorized callback error
    #[cfg(feature = "nois")]
    #[error("Nois callback was not from proxy contract: expected {proxy_addr}, was {caller}")]
    UnauthorizedNoisCallback {
        caller: cosmwasm_std::Addr,
        proxy_addr: cosmwasm_std::Addr,
    },

    #[cfg(feature = "nois")]
    #[error("Randomness already set for job_id {0}")]
    RandomnessAlreadySet(String)
}
