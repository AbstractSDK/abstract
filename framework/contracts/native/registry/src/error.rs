use abstract_std::{
    objects::{module::ModuleInfo, namespace::Namespace, validation::ValidationError, AccountId},
    AbstractError, ACCOUNT,
};
use cosmwasm_std::{Addr, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum RegistryError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    Validation(#[from] ValidationError),

    #[error("{0}")]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error(
        "Caller with info {} has code_id {} but expected {}",
        account_info,
        actual_code_id,
        expected_code_id
    )]
    NotAccountCodeId {
        account_info: ModuleInfo,
        expected_code_id: u64,
        actual_code_id: u64,
    },

    #[error("Caller has info {} but should be {}", caller_info, ACCOUNT)]
    NotAccountInfo { caller_info: ModuleInfo },

    #[error("Module {0} does not have a stored module reference")]
    ModuleNotFound(ModuleInfo),

    #[error("Module {0} cannot be updated")]
    NotUpdateableModule(ModuleInfo),

    #[error("Account ID {} is not in registry register", id)]
    UnknownAccountId { id: AccountId },

    #[error("Namespace {} is not in registry register", namespace)]
    UnknownNamespace { namespace: Namespace },

    #[error("Account owner mismatch sender: {}, owner: {}", sender, owner)]
    AccountOwnerMismatch { sender: Addr, owner: Addr },

    #[error("Account with ID {} has no owner", account_id)]
    NoAccountOwner { account_id: AccountId },

    #[error("Namespace {} is already occupied by account {}", namespace, id)]
    NamespaceOccupied { namespace: String, id: AccountId },

    #[error("Exceeds namespace limit: {}, current: {}", limit, current)]
    ExceedsNamespaceLimit { limit: usize, current: usize },

    #[error("The admin of an adapter must be None")]
    AdminMustBeNone,

    #[error("No action specified")]
    NoAction,

    #[error("Account {0} already exists")]
    AccountAlreadyExists(AccountId),

    #[error("Initialization funds can only be specified for apps and standalone modules")]
    RedundantInitFunds {},

    #[error("Sender {0} is not the IBC host {1}")]
    SenderNotIbcHost(String, String),

    #[error("requested sequence is invalid. Expected: {expected}, actual: {actual}")]
    InvalidAccountSequence { expected: u32, actual: u32 },
}

impl From<semver::Error> for RegistryError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
