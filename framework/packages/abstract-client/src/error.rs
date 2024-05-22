//! # Represents Abstract Client Errors

use abstract_interface::AbstractInterfaceError;
use abstract_std::{
    objects::{chain_name::ChainName, validation::ValidationError, AccountId},
    AbstractError,
};
use cosmwasm_std::Addr;
use thiserror::Error;

#[derive(Error, Debug)]
/// Error type for the abstract client crate.
#[allow(missing_docs)] // Error type names should be self-explanatory
pub enum AbstractClientError {
    #[error("{0}")]
    Abstract(#[from] AbstractError),

    #[error("{0}")]
    Interface(#[from] AbstractInterfaceError),

    #[error("{0}")]
    CwOrch(#[from] cw_orch::prelude::CwOrchError),

    #[error("{0}")]
    Semver(#[from] semver::Error),

    #[error("{0}")]
    ValidationError(#[from] ValidationError),

    #[error("Module not installed")]
    ModuleNotInstalled {},

    #[error("Account is Renounced and does not have an owner.")]
    RenouncedAccount {},

    #[error("Can't retrieve Account for unclaimed namespace \"{namespace}\".")]
    NamespaceNotClaimed { namespace: String },

    #[error("Can't add custom funds when using auto_fund.")]
    FundsWithAutoFund {},

    #[error("Account creation auto_fund assertion failed with required funds: {0:?}")]
    AutoFundsAssertFailed(Vec<cosmwasm_std::Coin>),

    #[error("Use build_remote for remote accounts")]
    RemoteAccountOnBuild {},

    #[error("Use build for non-remote accounts")]
    LocalAccountOnBuildRemote {},

    #[error("Remote account of {account_id} not found on {chain} in {ibc_client_addr}")]
    RemoteAccountNotFound {
        account_id: AccountId,
        chain: ChainName,
        ibc_client_addr: Addr,
    },
}
