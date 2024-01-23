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

pub use account::{Account, AccountBuilder};
pub use application::Application;
pub use builder::AbstractClientBuilder;
pub use client::AbstractClient;
pub use error::AbstractClientError;
pub use infrastructure::Environment;
pub use publisher::{Publisher, PublisherBuilder};

// Re-export common used types
pub use abstract_core::objects::gov_type::GovernanceDetails;
pub use abstract_core::objects::namespace::Namespace;

// Re-export `ClientResolve` trait
pub use abstract_interface::ClientResolve;
