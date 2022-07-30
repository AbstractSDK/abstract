//! # Abstract SDK
//!
//! An SDK for writing Abstract OS smart-contracts.
//!
//! ## Description
//! The internal lay-out and state management of Abstract OS allows smart-contract engineers to write deployment-generic code.
//! The functions provided by this SDK can be used to quickly write and test your unique CosmWasm application.

pub mod _modules;
mod api;
pub mod common_namespace;
pub mod cw20;
pub mod manager;
mod module_traits;
pub mod proxy;
pub mod tendermint_staking;
pub mod version_control;
pub mod memory {
    pub use abstract_os::objects::memory::Memory;
}

pub use api::{api_req, configure_api};
pub use module_traits::{LoadMemory, OsExecute};

pub extern crate abstract_os;
