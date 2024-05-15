use abstract_sdk::AbstractSdkError;
use abstract_std::{
    objects::{version_control::VersionControlError, AccountId},
    AbstractError,
};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use cw_asset::AssetError;
use cw_controllers::AdminError;
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
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("{0}")]
    VersionControlError(#[from] VersionControlError),

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),

    #[error("module {0} is required to be of kind {1}")]
    WrongModuleKind(String, String),

    #[error("Your payment does not match the required payment {0}")]
    WrongAmount(String),

    #[error("No payment received")]
    NoPaymentReceived {},

    #[error("Can not create remote accounts without configured IBC host.")]
    IbcHostNotSet {},

    #[error("A trace must exist of at least one or at most {0} hops but has {1}")]
    InvalidTrace(usize, usize),

    #[error("Sender {0} is not the IBC host {1}")]
    SenderNotIbcHost(String, String),

    #[error("The caller ({caller}) is not the owner account's manager ({manager}). Only manager can create sub-accounts for its account.", )]
    SubAccountCreatorNotManager { caller: String, manager: String },

    #[error("Expected local account id doesn't match, expected: {predicted}, actual: {actual} Try again.")]
    ExpectedAccountIdFailed {
        predicted: AccountId,
        actual: AccountId,
    },

    #[error("NotSetup")]
    NotSetup {},
    
    #[error("AlreadyBeingSetup")]
    AlreadyBeingSetup {},

    #[error("AlreadySetup")]
    AlreadySetup {},

    #[error("Invalid Bitsong name")]
    InvalidName {},

    #[error("Name too short")]
    NameTooShort {},

    #[error("Name too long")]
    NameTooLong {},
}
