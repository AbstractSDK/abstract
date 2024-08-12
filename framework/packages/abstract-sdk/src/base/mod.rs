//! Base of an Abstract Module and its features.  
//!
//! Is used by the `abstract-adapter`, `abstract-ibc-host` and `abstract-app` crates.

mod contract_base;
mod endpoints;
pub(crate) mod features;
mod handler;

pub use contract_base::{
    AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
    MigrateHandlerFn, ModuleIbcHandlerFn, ModuleId, ModuleMetadata, QueryHandlerFn, ReplyHandlerFn,
    SudoHandlerFn, VersionString,
};
pub use endpoints::{
    ExecuteEndpoint, IbcCallbackEndpoint, InstantiateEndpoint, MigrateEndpoint, ModuleIbcEndpoint,
    QueryEndpoint, ReplyEndpoint, SudoEndpoint,
};
pub use handler::Handler;
