use abstract_sdk::std::objects::module::ModuleInfo;
use abstract_std::{
    objects::{registry::RegistryError, validation::ValidationError},
    AbstractError,
};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AccountError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error(transparent)]
    Ownership(#[from] abstract_std::objects::ownership::GovOwnershipError),

    #[error(transparent)]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error(transparent)]
    RegistryError(#[from] RegistryError),

    #[error("Your account is currently suspended")]
    AccountSuspended {},

    // ** Modules Error ** //
    #[error("Failed to query modules to install: {error}")]
    QueryModulesFailed { error: RegistryError },

    #[error("Module with id: {0} is already installed")]
    ModuleAlreadyInstalled(String),

    #[error("Reinstalls of same version of app or standalone are not allowed")]
    ProhibitedReinstall {},

    #[error("The provided module {0} can't be installed on an Abstract account")]
    ModuleNotInstallable(String),

    #[error("The name of the proposed module can not have length 0.")]
    InvalidModuleName {},

    #[error("The provided module {0} was not found")]
    ModuleNotFound(String),

    #[error("Cannot migrate {} twice", module_id)]
    DuplicateModuleMigration { module_id: String },

    #[error("{0} not upgradable")]
    NotUpgradeable(ModuleInfo),

    #[error("Cannot remove module because {0:?} depend(s) on it.")]
    ModuleHasDependents(Vec<String>),

    #[error("Module {module_id} with version {version} does not fit requirement {comp}, post_migration: {post_migration}")]
    VersionRequirementNotMet {
        module_id: String,
        version: String,
        comp: String,
        post_migration: bool,
    },

    #[error("module {0} is a dependency of {1} and is not installed.")]
    DependencyNotMet(String, String),

    #[error("Max amount of modules registered")]
    ModuleLimitReached {},

    #[error("Module with address {0} is already whitelisted")]
    AlreadyWhitelisted(String),

    #[error("can't remove module that is not whitelisted")]
    NotWhitelisted {},

    // ** Sub Account ** //
    #[error("Removing sub account failed")]
    SubAccountRemovalFailed {},

    #[error("Register of sub account failed")]
    SubAccountRegisterFailed {},

    #[error("Can't renounce account, with active sub account")]
    RenounceWithSubAccount {},

    // ** Other Errors ** //
    #[error("No updates were included")]
    NoUpdates {},

    #[error("invalid configuration action, {}", error)]
    InvalidConfigAction { error: StdError },

    #[error("The provided contract version {0} is lower than the current version {1}")]
    OlderVersion(String, String),

    #[error("Sender is not whitelisted and is not a valid owner")]
    SenderNotWhitelistedOrOwner {},

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),

    #[error("The caller ({caller}) is not the owner account's account ({account}). Only account can create sub-accounts for itself.", )]
    SubAccountCreatorNotAccount { caller: String, account: String },

    #[error("You can't chain admin calls")]
    CantChainAdminCalls {},

    #[error("Abstract Account Address don't match to the Contract address")]
    AbsAccInvalidAddr {
        abstract_account: String,
        contract: String,
    },

    #[error("Abstract Account don't have Authentication")]
    AbsAccNoAuth {},

    #[cfg(feature = "xion")]
    #[error(transparent)]
    AbstractXion(#[from] abstract_xion::error::ContractError),
}
