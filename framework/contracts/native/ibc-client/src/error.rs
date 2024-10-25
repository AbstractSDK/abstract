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
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    AbstractSdk(#[from] AbstractSdkError),

    #[error(transparent)]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error(transparent)]
    RegistryError(#[from] RegistryError),

    #[error(transparent)]
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
