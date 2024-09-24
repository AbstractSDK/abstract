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
mod service;
pub(crate) mod source;

#[cfg(feature = "interchain")]
mod interchain;

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

#[cfg(feature = "interchain")]
pub use interchain::*;
