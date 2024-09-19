use abstract_sdk::{std::objects::module::ModuleInfo, AbstractSdkError};
use abstract_std::{
    objects::{validation::ValidationError, version_control::VersionControlError},
    AbstractError,
};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AccountError {
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

    #[error("Your account is currently suspended")]
    AccountSuspended {},

    // ** Modules Error ** //
    #[error("Failed to query modules to install: {error}")]
    QueryModulesFailed { error: VersionControlError },

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
    ModuleLimitReached,

    #[error("Module with address {0} is already whitelisted")]
    AlreadyWhitelisted(String),

    #[error("Module with address {0} not found in whitelist")]
    NotWhitelisted(String),

    // ** Sub Account ** //
    #[error("Removing sub account failed")]
    SubAccountRemovalFailed {},

    #[error("Register of sub account failed")]
    SubAccountRegisterFailed {},

    #[error("Can't renounce account, with active sub account")]
    RenounceWithSubAccount {},

    // ** Other Errors TODO: sort ? ** //
    #[error("No updates were included")]
    NoUpdates {},

    #[error("invalid configuration action, {}", error)]
    InvalidConfigAction { error: StdError },

    #[error("The provided contract version {0} is lower than the current version {1}")]
    OlderVersion(String, String),

    #[error("Sender is not whitelisted")]
    SenderNotWhitelisted {},

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),

    #[error("The caller ({caller}) is not the owner account's account ({account}). Only account can create sub-accounts for itself.", )]
    SubAccountCreatorNotAccount { caller: String, account: String },

    // TODO: Feature flag xion
    #[error(transparent)]
    EncodeError(#[from] cosmos_sdk_proto::prost::EncodeError),

    #[error(transparent)]
    DecodeError(#[from] cosmos_sdk_proto::prost::DecodeError),

    #[error(transparent)]
    Verification(#[from] cosmwasm_std::VerificationError),

    #[error(transparent)]
    FromHex(#[from] hex::FromHexError),

    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),

    #[error(transparent)]
    Rsa(#[from] rsa::Error),

    #[error(transparent)]
    P256EllipticCurve(#[from] p256::elliptic_curve::Error),

    #[error(transparent)]
    P256EcdsaCurve(#[from] p256::ecdsa::Error),

    #[error(transparent)]
    RecoverPubkey(#[from] cosmwasm_std::RecoverPubkeyError),

    #[error("The pubkey recovered from the signature does not match")]
    RecoveredPubkeyMismatch {},

    #[error("Signature is empty")]
    EmptySignature {},

    #[error("Short signature")]
    ShortSignature {},

    #[error("Signature is invalid")]
    InvalidSignature {},

    #[error("Signature is invalid. expected: {expected}, received {received}")]
    InvalidSignatureDetail { expected: String, received: String },

    #[error("Recovery id can only be one of 0, 1, 27, 28")]
    InvalidRecoveryId {},

    #[error("Invalid token")]
    InvalidToken {},

    #[error("url parse error: {url}")]
    URLParse { url: String },

    #[error("cannot override existing authenticator at index {index}")]
    OverridingIndex { index: u8 },

    #[error(transparent)]
    FromUTF8(#[from] std::string::FromUtf8Error),
}
