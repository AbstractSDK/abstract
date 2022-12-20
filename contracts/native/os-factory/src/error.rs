use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum OsFactoryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),
    #[error("module {0} is required to be of kind {1}")]
    WrongModuleKind(String, String),

    #[error("Bad subscription module configuration.")]
    UnsupportedAsset(),

    #[error("Your payment does not match the required payment {0}")]
    WrongAmount(String),

    #[error("No payment received")]
    NoPaymentReceived {},
}
