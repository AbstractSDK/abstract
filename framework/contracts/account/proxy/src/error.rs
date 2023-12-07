use abstract_core::{objects::ans_host::AnsHostError, AbstractError};
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::{StdError, Uint128};
use cw_asset::AssetError;
use cw_utils::ParseReplyError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ProxyError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error(transparent)]
    Admin(#[from] ::cw_controllers::AdminError),

    #[error("{0}")]
    Parse(#[from] ParseReplyError),

    #[error("{0}")]
    AnsHostError(#[from] AnsHostError),

    #[error("Module with address {0} is already whitelisted")]
    AlreadyWhitelisted(String),

    #[error("Module with address {0} not found in whitelist")]
    NotWhitelisted(String),

    #[error("Sender is not whitelisted")]
    SenderNotWhitelisted {},

    #[error("Max amount of assets registered")]
    AssetsLimitReached,

    #[error("Max amount of modules registered")]
    ModuleLimitReached,

    #[error("no base asset registered on proxy")]
    MissingBaseAsset,

    #[error("The proposed update resulted in a bad configuration: {0}")]
    BadUpdate(String),

    #[error(
        "Account balance too low, {} requested but it only has {}",
        requested,
        balance
    )]
    Broke {
        balance: Uint128,
        requested: Uint128,
    },

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),
}
