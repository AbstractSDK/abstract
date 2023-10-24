pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod account;
#[cfg(feature = "daemon")]
mod migrate;

pub use crate::account::*;

mod native;

pub use crate::native::*;

mod interfaces;

pub use crate::interfaces::*;

mod deployers;
mod deployment;
mod error;

pub use error::AbstractInterfaceError;

pub use crate::deployers::*;

pub use crate::deployment::*;
