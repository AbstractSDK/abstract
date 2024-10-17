use abstract_std::{objects::registry::RegistryError, AbstractError};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ModuleFactoryError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error(transparent)]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error(transparent)]
    RegistryError(#[from] RegistryError),

    #[error("This module type can not be installed on your Account")]
    ModuleNotInstallable {},
}
