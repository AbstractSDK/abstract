//! Base of an Abstract Module and its features.  
//!
//! Is used by the `abstract-adapter`, `abstract-ibc-host` and `abstract-app` crates.

mod contract_base;
mod endpoints;
pub(crate) mod features;
mod handler;

pub use contract_base::{
    AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
    MigrateHandlerFn, QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn, SudoHandlerFn,
    VersionString,
};
pub use endpoints::{
    migrate::MigrateEndpoint, ExecuteEndpoint, IbcCallbackEndpoint, InstantiateEndpoint,
    QueryEndpoint, ReceiveEndpoint, ReplyEndpoint, SudoEndpoint,
};
pub use handler::Handler;
