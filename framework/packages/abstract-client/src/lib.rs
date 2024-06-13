#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub(crate) mod account;
mod application;
pub mod builder;
mod client;
mod error;
pub(crate) mod infrastructure;
#[cfg(feature = "test-utils")]
mod mut_client;
mod publisher;
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
pub use source::AccountSource;

// Interchain stuff
#[cfg(feature = "interchain")]
mod interchain {
    pub(crate) mod remote_account;
    mod remote_application;
    pub use remote_account::RemoteAccount;
    pub use remote_application::RemoteApplication;
}
#[cfg(feature = "interchain")]
pub use interchain::*;
