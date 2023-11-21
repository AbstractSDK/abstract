use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AppError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    Admin(#[from] AdminError),
}
