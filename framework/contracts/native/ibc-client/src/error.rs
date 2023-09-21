use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use polytone::callbacks::CallbackMessage;
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
    Admin(#[from] AdminError),

    #[error("No account for chain {0}")]
    UnregisteredChain(String),

    #[error("remote account changed from {old} to {addr}")]
    RemoteAccountChanged { addr: String, old: String },

    #[error("packages that contain internal calls are not allowed")]
    ForbiddenInternalCall {},

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
}
