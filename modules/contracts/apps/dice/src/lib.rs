pub mod contract;
pub mod error;
mod handlers;
pub mod msg;
pub mod state;

#[cfg(feature = "interface")]
pub use contract::interface::DiceApp;
#[cfg(feature = "interface")]
pub use msg::{DiceExecuteMsgFns, DiceQueryMsgFns};
