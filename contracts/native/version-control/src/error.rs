use abstract_core::{objects::AccountId, AbstractError};
use abstract_sdk::{core::objects::module::ModuleInfo, AbstractSdkError};
use cosmwasm_std::StdError;
use cw_controllers::AdminError;
use thiserror::Error;

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
    UnknownNamespace { namespace: String },

    #[error("Account owner mismatch sender: {}, owner: {}", sender, owner)]
    AccountOwnerMismatch { sender: String, owner: String },

    #[error("Namespace {} is already occupied by {}", namespace, id)]
    NamespaceOccupied { namespace: String, id: AccountId },

    #[error("Exceeds namespace limit: {}, current: {}", limit, current)]
    ExceedsNamespaceLimit { limit: usize, current: usize },

    #[error(
        "Decrease namespace limit not allowed: {}, current: {}",
        limit,
        current
    )]
    DecreaseNamespaceLimit { limit: u32, current: u32 },

    #[error("As namespace owner you can only yank a module, not remove it.")]
    OnlyYankAllowed,

    #[error("No action specified")]
    NoAction,
}

impl From<cw_semver::Error> for VCError {
    fn from(err: cw_semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
