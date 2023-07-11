pub mod contract;
pub mod error;
mod handlers;
pub mod msg;
pub mod state;

#[cfg(feature = "interface")]
pub use contract::interface::DCAApp;
#[cfg(feature = "interface")]
pub use msg::{DCAExecuteMsgFns, DCAQueryMsgFns};
