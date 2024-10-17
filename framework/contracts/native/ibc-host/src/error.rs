use abstract_std::{
    objects::{ans_host::AnsHostError, registry::RegistryError},
    AbstractError,
};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum HostError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    OwnershipError(#[from] OwnershipError),

    #[error(transparent)]
    RegistryError(#[from] RegistryError),

    #[error(transparent)]
    AnsHostError(#[from] AnsHostError),

    #[error(transparent)]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Chain or account address already registered.")]
    ProxyAddressExists {},

    #[error("Can't send a module-to-module packet to {0}, wrong module type")]
    WrongModuleAction(String),
}

impl From<semver::Error> for HostError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
