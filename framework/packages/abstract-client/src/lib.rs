#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
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

pub use abstract_interface::ClientResolve;
pub use abstract_std::objects::{gov_type::GovernanceDetails, namespace::Namespace};
pub use account::{Account, AccountBuilder};
pub use application::Application;
pub use builder::AbstractClientBuilder;
pub use client::AbstractClient;
pub use error::AbstractClientError;
pub use infrastructure::Environment;
pub use publisher::Publisher;
pub use service::Service;
pub use source::AccountSource;

#[cfg(feature = "interchain")]
pub use interchain::*;
