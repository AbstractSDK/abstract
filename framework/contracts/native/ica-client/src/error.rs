use abstract_std::{
    objects::{ans_host::AnsHostError, registry::RegistryError},
    AbstractError,
};
use cosmwasm_std::StdError;
use thiserror::Error;

// TODO: Remove unused errs
#[derive(Error, Debug, PartialEq)]
pub enum IcaClientError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error("{0}")]
    RegistryError(#[from] RegistryError),

    #[error("{0}")]
    AnsHostError(#[from] AnsHostError),

    #[error("chain {chain} has no associated type (evm/cosmos/...)")]
    NoChainType { chain: String },

    #[error("No existing remote account and no recipient specified")]
    NoRecipient {},

    #[error("messages for chain {chain} are not of type {ty}")]
    WrongChainType { chain: String, ty: String },
}
