//! Base of an Abstract Module and its features.  
//!
//! Is used by the `abstract-adapter`, `abstract-ibc-host` and `abstract-app` crates.

mod contract_base;
mod endpoints;
pub(crate) mod features;
mod handler;

#[cfg(feature = "module-ibc")]
pub use contract_base::ModuleIbcHandlerFn;
pub use contract_base::{
    AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
    MigrateHandlerFn, QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn, SudoHandlerFn,
    VersionString,
};
#[cfg(feature = "module-ibc")]
pub use endpoints::ModuleIbcEndpoint;
pub use endpoints::{
    ExecuteEndpoint, IbcCallbackEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint,
    ReceiveEndpoint, ReplyEndpoint, SudoEndpoint,
};
pub use handler::Handler;
