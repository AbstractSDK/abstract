use cosmwasm_std::Coin;
use cosmwasm_std::{Addr, StdError};
use cw_controllers::AdminError;
use thiserror::Error;

use abstract_core::objects::namespace::Namespace;
use abstract_core::{objects::AccountId, AbstractError};
use abstract_sdk::{core::objects::module::ModuleInfo, AbstractSdkError};

#[derive(Error, Debug, PartialEq)]
pub enum VCError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("{0}")]
    Ownership(#[from] cw_ownable::OwnershipError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Module {0} does not have a stored module reference")]
    ModuleNotFound(ModuleInfo),

    #[error("Module {0} is in both approve and reject")]
    InvalidApproveList(ModuleInfo),

    #[error("Module {0} cannot be updated")]
    NotUpdateableModule(ModuleInfo),

    #[error("Account ID {} is not in version control register", id)]
    UnknownAccountId { id: AccountId },

    #[error("Namespace {} is not in version control register", namespace)]
    UnknownNamespace { namespace: Namespace },

    #[error("Account owner mismatch sender: {}, owner: {}", sender, owner)]
    AccountOwnerMismatch { sender: Addr, owner: Addr },

    #[error("Account with ID {} has no owner", account_id)]
    NoAccountOwner { account_id: AccountId },

    #[error("Namespace {} is already occupied by account {}", namespace, id)]
    NamespaceOccupied { namespace: String, id: AccountId },

    #[error("Exceeds namespace limit: {}, current: {}", limit, current)]
    ExceedsNamespaceLimit { limit: usize, current: usize },

    #[error(
        "Decreasing namespace limit is not allowed: {}, current: {}",
        limit,
        current
    )]
    DecreaseNamespaceLimit { limit: u32, current: u32 },

    #[error("As namespace owner you can only yank a module, not remove it.")]
    OnlyYankAllowed,

    #[error("The admin of an adapter must be None")]
    AdminMustBeNone,

    #[error("No action specified")]
    NoAction,

    #[error("Invalid fee payment sent. Expected {}, sent {:?}", expected, sent)]
    InvalidFeePayment { expected: Coin, sent: Vec<Coin> },
}

impl From<cw_semver::Error> for VCError {
    fn from(err: cw_semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
