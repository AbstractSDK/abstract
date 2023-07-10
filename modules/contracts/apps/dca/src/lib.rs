pub mod contract;
pub mod error;
mod handlers;
pub mod msg;
mod replies;
pub mod state;

#[cfg(feature = "interface")]
pub use contract::interface::DCAApp;
#[cfg(feature = "interface")]
pub use msg::{DCAExecuteMsgFns, DCAQueryMsgFns};
