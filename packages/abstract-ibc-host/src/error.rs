use abstract_os::AbstractOsError;
use abstract_sdk::{os::abstract_ica::SimpleIcaError, AbstractSdkError};
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum HostError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AbstractOs(#[from] AbstractOsError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("This host does not implement any custom queries")]
    NoCustomQueries,

    #[error("{0}")]
    AdminError(#[from] AdminError),

    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),

    #[error("{0}")]
    SimpleIca(#[from] SimpleIcaError),

    #[error("Cannot register over an existing channel")]
    ChannelAlreadyRegistered,

    #[error("Invalid reply id")]
    InvalidReplyId,

    #[error("This channel has not been closed.")]
    ChannelNotClosed,

    #[error("A valid proxy address must be provided.")]
    MissingProxyAddress,

    #[error("Missing target proxy to send messages to.")]
    NoTarget,

    #[error("Ibc hopping not supported")]
    IbcHopping,
}
