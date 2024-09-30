use abstract_sdk::{std::objects::module::ModuleInfo, AbstractSdkError};
use abstract_std::{
    objects::{validation::ValidationError, registry::RegistryError},
    AbstractError,
};
use cosmwasm_std::{Instantiate2AddressError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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
    ModuleLimitReached,

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

    // ** Other Errors TODO: sort ? ** //
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

    #[error("Abstract Account Address don't match to the Contract address")]
    AbsAccInvalidAddr {
        abstract_account: String,
        contract: String,
    },

    #[error("Abstract Account don't have Authentication")]
    AbsAccNoAuth {},

    #[cfg(feature = "xion")]
    #[error(transparent)]
    EncodeError(#[from] cosmos_sdk_proto::prost::EncodeError),

    #[cfg(feature = "xion")]
    #[error(transparent)]
    DecodeError(#[from] cosmos_sdk_proto::prost::DecodeError),

    #[cfg(feature = "xion")]
    #[error(transparent)]
    Verification(#[from] cosmwasm_std::VerificationError),

    #[cfg(feature = "xion")]
    #[error(transparent)]
    FromHex(#[from] hex::FromHexError),

    #[cfg(feature = "xion")]
    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[cfg(feature = "xion")]
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),

    #[cfg(feature = "xion")]
    #[error(transparent)]
    Rsa(#[from] rsa::Error),

    #[cfg(feature = "xion")]
    #[error(transparent)]
    P256EllipticCurve(#[from] p256::elliptic_curve::Error),

    // TODO: no PartialEq implemented for it, see `secp256r1.rs`
    // #[cfg(feature = "xion")]
    // #[error(transparent)]
    // P256EcdsaCurve(#[from] p256::ecdsa::Error),
    #[cfg(feature = "xion")]
    #[error(transparent)]
    RecoverPubkey(#[from] cosmwasm_std::RecoverPubkeyError),

    #[cfg(feature = "xion")]
    #[error("The pubkey recovered from the signature does not match")]
    RecoveredPubkeyMismatch {},

    #[cfg(feature = "xion")]
    #[error("Signature is empty")]
    EmptySignature {},

    #[cfg(feature = "xion")]
    #[error("Short signature")]
    ShortSignature {},

    #[cfg(feature = "xion")]
    #[error("Signature is invalid")]
    InvalidSignature {},

    #[cfg(feature = "xion")]
    #[error("Signature is invalid. expected: {expected}, received {received}")]
    InvalidSignatureDetail { expected: String, received: String },

    #[cfg(feature = "xion")]
    #[error("Recovery id can only be one of 0, 1, 27, 28")]
    InvalidRecoveryId {},

    #[cfg(feature = "xion")]
    #[error("Invalid token")]
    InvalidToken {},

    #[cfg(feature = "xion")]
    #[error("url parse error: {url}")]
    URLParse { url: String },

    #[cfg(feature = "xion")]
    #[error("cannot override existing authenticator at index {index}")]
    OverridingIndex { index: u8 },

    #[cfg(feature = "xion")]
    #[error("cannot delete the last authenticator")]
    MinimumAuthenticatorCount {},

    #[cfg(feature = "xion")]
    #[error("Authenticator id should be in range from 0 to 127")]
    TooBigAuthId {},

    #[cfg(feature = "xion")]
    #[error(transparent)]
    FromUTF8(#[from] std::string::FromUtf8Error),
}
