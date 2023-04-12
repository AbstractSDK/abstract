use abstract_core::AbstractError;
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::StdError;
use cw_asset::AssetError;
use cw_controllers::AdminError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AccountFactoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Asset(#[from] AssetError),

    #[error("{0}")]
    Ownership(#[from] OwnershipError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),

    #[error("module {0} is required to be of kind {1}")]
    WrongModuleKind(String, String),

    #[error("Your payment does not match the required payment {0}")]
    WrongAmount(String),

    #[error("No payment received")]
    NoPaymentReceived {},
}
