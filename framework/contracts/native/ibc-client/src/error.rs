use abstract_sdk::AbstractSdkError;
use abstract_std::{
    ibc::polytone_callbacks::CallbackMessage,
    objects::{ans_host::AnsHostError, registry::RegistryError, AccountId},
    AbstractError,
};
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum IbcClientError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error("{0}")]
    RegistryError(#[from] RegistryError),

    #[error("{0}")]
    AnsHostError(#[from] AnsHostError),

    #[error("No account for chain {0}")]
    UnregisteredChain(String),

    #[error("Calling internal actions externally is not allowed")]
    ForbiddenInternalCall {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("IBC Execution Failed, {0:?}")]
    IbcFailed(CallbackMessage),

    #[error("Chain or host address already registered.")]
    HostAddressExists {},

    #[error("IBC Client is not installed on {account_id}")]
    IbcClientNotInstalled { account_id: AccountId },
}
