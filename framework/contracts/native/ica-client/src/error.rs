use abstract_std::{
    objects::{ans_host::AnsHostError, registry::RegistryError},
    AbstractError,
};
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum IcaClientError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error(transparent)]
    RegistryError(#[from] RegistryError),

    #[error(transparent)]
    AnsHostError(#[from] AnsHostError),

    #[error("chain {chain} has no associated type (evm/cosmos/...)")]
    NoChainType { chain: String },

    #[error("No existing remote account and no recipient specified")]
    NoRecipient {},

    #[error("messages for chain {chain} are not of type {ty}")]
    WrongChainType { chain: String, ty: String },
}
