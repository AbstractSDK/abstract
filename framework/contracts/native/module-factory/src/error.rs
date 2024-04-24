use abstract_sdk::AbstractSdkError;
use abstract_std::{objects::version_control::VersionControlError, AbstractError};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use cw_asset::AssetError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ModuleFactoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("{0}")]
    VersionControlError(#[from] VersionControlError),

    #[error("Calling contract is not a registered Account Manager")]
    UnknownCaller(),

    #[error("Reply ID does not match any known Reply ID")]
    UnexpectedReply(),

    #[error("This module type can not be installed on your Account")]
    ModuleNotInstallable {},
}
