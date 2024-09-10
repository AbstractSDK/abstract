use abstract_sdk::AbstractSdkError;
use abstract_std::{
    objects::{validation::ValidationError, version_control::VersionControlError},
    AbstractError,
};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AccountError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Validation(#[from] ValidationError),

    #[error("{0}")]
    Ownership(#[from] abstract_std::objects::ownership::GovOwnershipError),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("{0}")]
    VersionControlError(#[from] VersionControlError),

    #[error("{0}")]
    Manager(#[from] manager::error::ManagerError),

    #[error("{0}")]
    Proxy(#[from] proxy::error::ProxyError),

    #[error("Your account is currently suspended")]
    AccountSuspended {},

    #[error("The caller ({caller}) is not the owner account's account ({account}). Only account can create sub-accounts for itself.", )]
    SubAccountCreatorNotAccount { caller: String, account: String },
}
