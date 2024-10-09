// #![doc = include_str!("../README.md")]
#![warn(missing_docs)]
pub(crate) mod account;
mod application;
pub mod builder;
mod client;
mod error;
pub(crate) mod infrastructure;
#[cfg(feature = "test-utils")]
mod mut_client;
mod publisher;
mod service;
pub(crate) mod source;

// Re-export common used types
pub use abstract_std::objects::{gov_type::GovernanceDetails, namespace::Namespace};
// Re-export `ClientResolve` trait
pub use abstract_interface::ClientResolve;
pub use account::{Account, AccountBuilder};
pub use application::Application;
pub use builder::AbstractClientBuilder;
pub use client::AbstractClient;
pub use error::AbstractClientError;
pub use infrastructure::Environment;
pub use publisher::{Publisher, PublisherBuilder};
pub use service::Service;
pub use source::AccountSource;

// Interchain stuff
#[cfg(feature = "interchain")]
mod interchain {
    pub(crate) mod remote_account;
    mod remote_application;
    pub use remote_account::RemoteAccount;
    pub use remote_application::RemoteApplication;

    /// IbcTxAnalysis after waiting for interchain action
    pub struct IbcTxAnalysisV2<Chain: cw_orch::environment::CwEnv>(
        pub cw_orch_interchain::types::IbcTxAnalysis<Chain>,
    );
}
#[cfg(feature = "interchain")]
pub use interchain::*;
