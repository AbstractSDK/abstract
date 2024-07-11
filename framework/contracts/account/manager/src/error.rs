use abstract_sdk::{std::objects::module::ModuleInfo, AbstractSdkError};
use abstract_std::{
    objects::{validation::ValidationError, version_control::VersionControlError},
    AbstractError,
};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ManagerError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Validation(#[from] ValidationError),

    #[error("{0}")]
    Ownership(#[from] abstract_std::objects::ownership::GovOwnershipError),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("{0}")]
    VersionControlError(#[from] VersionControlError),

    #[error("Module with id: {0} is already installed")]
    ModuleAlreadyInstalled(String),

    #[error("Cannot remove module because {0:?} depend(s) on it.")]
    ModuleHasDependents(Vec<String>),

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),

    #[error("The name of the proposed module can not have length 0.")]
    InvalidModuleName {},

    #[error("Registering module fails because caller is not module factory")]
    CallerNotModuleFactory {},

    #[error("A migrate msg is required when when migrating this module")]
    MsgRequired {},

    #[error("{0} not upgradable")]
    NotUpgradeable(ModuleInfo),

    #[error("Cannot migrate {} twice", module_id)]
    DuplicateModuleMigration { module_id: String },

    #[error("Your account is currently suspended")]
    AccountSuspended {},

    #[error("The provided contract version {0} is lower than the current version {1}")]
    OlderVersion(String, String),

    #[error("The provided module {0} was not found")]
    ModuleNotFound(String),

    #[error("The provided module {0} can't be installed on an Abstract account")]
    ModuleNotInstallable(String),

    #[error("Module {module_id} with version {version} does not fit requirement {comp}, post_migration: {post_migration}")]
    VersionRequirementNotMet {
        module_id: String,
        version: String,
        comp: String,
        post_migration: bool,
    },

    #[error("module {0} is a dependency of {1} and is not installed.")]
    DependencyNotMet(String, String),

    #[error("The provided module {0} has an invalid module reference.")]
    InvalidReference(ModuleInfo),

    #[error("Cannot remove proxy")]
    CannotRemoveProxy {},

    #[error("No updates were included")]
    NoUpdates {},

    #[error("invalid configuration action, {}", error)]
    InvalidConfigAction { error: StdError },

    #[error("Must use ProposeOwner to change owner")]
    MustUseProposeOwner {},

    #[error("The address {0} doesn't have an owner, the manager can't determine admin right")]
    NoContractOwner(String),

    #[error("
            Checking the admin recursively failed. 
            You either have the an error in your sub-account configuration or you are not authorized to make this call.
    ")]
    SubAccountAdminVerification,

    #[error("Removing sub account failed")]
    SubAccountRemovalFailed {},

    #[error("Register of sub account failed")]
    SubAccountRegisterFailed {},

    #[error("Can't renounce account, with active sub account")]
    RenounceWithSubAccount {},

    #[error("Can't propose Renounced governance, use update_ownership instead")]
    ProposeRenounced {},

    #[error("Can't create account with Renounced governance")]
    InitRenounced {},

    #[error("Reinstalls of same version of app or standalone are not allowed")]
    ProhibitedReinstall {},

    #[error("Failed to query modules to install: {error}")]
    QueryModulesFailed { error: VersionControlError },
}
