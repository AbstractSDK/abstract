#![cfg(not(target_arch = "wasm32"))]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod account;
mod ibc;
#[cfg(feature = "daemon")]
mod migrate;

pub use crate::{account::*, ibc::*};

mod native;

pub use crate::native::*;

mod interfaces;

pub use crate::interfaces::*;

mod daemon_state;
mod deployers;
mod deployment;
mod error;

pub use error::AbstractInterfaceError;

pub use crate::{deployers::*, deployment::*};

pub use daemon_state::AbstractDaemonState;
