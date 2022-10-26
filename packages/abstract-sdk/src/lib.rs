#![feature(generic_associated_types)]
//! # Abstract SDK
//!
//! An SDK for writing Abstract OS smart-contracts.
//!
//! ## Description
//! The internal lay-out and state management of Abstract OS allows smart-contract engineers to write deployment-generic code.
//! The functions provided by this SDK can be used to quickly write and test your unique CosmWasm application.
pub type OsAction<T = Empty> = CosmosMsg<T>;

pub mod api;
pub mod cw20;
mod exchange;
mod ibc_client;
pub mod manager;
mod memory_traits;
mod module_traits;
pub mod proxy;
pub mod tendermint_staking;
mod traits;
mod version_control;
pub mod memory {
    pub use abstract_os::objects::memory::Memory;
}

pub use abstract_os::{
    objects::common_namespace::{ADMIN, BASE_STATE, CONTRACT_VERSION},
    registry::*,
};
pub use api::{api_request, configure_api};
use cosmwasm_std::{CosmosMsg, Empty};
pub use ibc_client::{host_ibc_action, ics20_transfer};
pub use manager::{query_module_address, query_module_version};
pub use memory_traits::Resolve;
pub use module_traits::{Dependency, MemoryOperation, OsExecute};
pub use proxy::{os_module_action, query_total_value};
pub use traits::{
    execute::AbstractExecute,
    ibc_callback::{IbcCallbackEndpoint, IbcCallbackHandlerFn},
    receive::{ReceiveEndpoint, ReceiveHandlerFn},
};
pub use version_control::{get_module, get_os_core, verify_os_manager, verify_os_proxy};
pub extern crate abstract_os;
