// #![warn(missing_docs)]

// https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html

//! # Abstract OS
//!
//! Abstract OS is the interface-defining crate to the Abstract OS smart-contract framework.
//!
//! ## Description
//! This crate provides the key utilities that are required to integrate with or write Abstract contracts.
//!
//! ## Messages
//! All interfacing message structs are defined here so they can be imported.  
//! ```no_run
//! use abstract_os::manager::ExecuteMsg;
//! ```  
//! ### Assets
//! [`cw-asset`](https://crates.io/crates/cw-asset) is used for asset-management.
//! If a message requests a String value for an Asset field then you need to provide the human-readable memory key.  
//! The full list of supported assets and contracts is given [here](https://github.com/Abstract-OS/scripts/tree/main/resources/memory).  
//! The contract will handel address retrieval internally.  
//!
//! ## State
//! The internal state for each contract is also contained within this crate. This ensures that breaking changes to the internal state are easily spotted.
//! It also allows for tight and low-gas integration between contracts by performing raw queries on these states.
//! A contract's state object can be imported and used like:
//! ```ignore
//! use crate::manager::state::OS_ID
//! let os_id = OS_ID.query(querier, manager_address).unwrap();
//! ```
//! The internally stored objects are also contained within this package in [`crate::objects`].
//!
//! ## Names
//! Abstract contract names are used internally and for version management.
//! They are exported for ease of use:
//! ```no_run
//! use abstract_os::PROXY;
//! ```

pub use registry::*;

pub mod abstract_token;
pub mod add_on;
pub mod api;
pub mod dex;
pub mod liquidity_interface;
pub mod manager;
pub mod memory;
pub mod module_factory;
pub mod objects;
pub mod os_factory;
pub mod proxy;
pub(crate) mod registry;
pub mod subscription;
pub mod tendermint_staking;
pub mod version_control;
