use abstract_sdk::AbstractSdkError;
use abstract_std::{
    ibc::polytone_callbacks::CallbackMessage,
    objects::{ans_host::AnsHostError, version_control::VersionControlError, AccountId},
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
    VersionControlError(#[from] VersionControlError),

    #[error("{0}")]
    AnsHostError(#[from] AnsHostError),

    #[error("No account for chain {0}")]
    UnregisteredChain(String),

    #[error("remote account changed from {old} to {addr}")]
    RemoteAccountChanged { addr: String, old: String },

    #[error("Calling internal actions externally is not allowed")]
    ForbiddenInternalCall {},

    #[error("A non-module package (native or accounts) cannot execute an ibc module call")]
    ForbiddenModuleCall {},

    #[error("The host you are trying to connect is already connected")]
    HostAlreadyExists {},

    #[error("Only authorized ports can connect to the contract on the remote chain")]
    UnauthorizedConnection {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("IBC Execution Failed, {0:?}")]
    IbcFailed(CallbackMessage),

    #[error("Chain or host address already registered.")]
    HostAddressExists {},

    #[error("IBC Client is not installed on {account_id}")]
    IbcClientNotInstalled { account_id: AccountId },
}
