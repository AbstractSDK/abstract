//! # Represents Abstract Client Errors

use abstract_interface::AbstractInterfaceError;
use abstract_std::{
    objects::{validation::ValidationError, AccountId},
    AbstractError,
};
use thiserror::Error;

#[derive(Error, Debug)]
/// Error type for the abstract client crate.
#[allow(missing_docs)] // Error type names should be self-explanatory
pub enum AbstractClientError {
    #[error(transparent)]
    Abstract(#[from] AbstractError),

    #[error(transparent)]
    Interface(#[from] AbstractInterfaceError),

    #[error(transparent)]
    CwOrch(#[from] cw_orch::prelude::CwOrchError),

    #[error(transparent)]
    Semver(#[from] semver::Error),

    #[error(transparent)]
    ValidationError(#[from] ValidationError),

    #[error("Module not installed")]
    ModuleNotInstalled {},

    #[error("Can't retrieve Account for unclaimed namespace \"{namespace}\".")]
    NamespaceNotClaimed { namespace: String },

    #[error("Namespace \"{namespace}\" already claimed by account {account_id}")]
    NamespaceClaimed {
        namespace: String,
        account_id: AccountId,
    },

    #[error("Account {account} doesn't have an associated namespace")]
    NoNamespace { account: AccountId },

    #[error("Can't add custom funds when using auto_fund.")]
    FundsWithAutoFund {},

    #[error("Account creation auto_fund assertion failed with required funds: {0:?}")]
    AutoFundsAssertFailed(Vec<cosmwasm_std::Coin>),

    #[cfg(feature = "interchain")]
    #[error("Remote account of {account_id} not found on {chain} in {ibc_client_addr}")]
    RemoteAccountNotFound {
        account_id: abstract_std::objects::AccountId,
        chain: abstract_std::objects::TruncatedChainId,
        ibc_client_addr: cosmwasm_std::Addr,
    },

    #[cfg(feature = "interchain")]
    #[error(transparent)]
    InterchainError(#[from] cw_orch_interchain::core::InterchainError),

    #[error("Service API only allows claiming service modules")]
    ExpectedService {},
}
