#![cfg(not(target_arch = "wasm32"))]
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

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

mod deployers;
mod deployment;
mod error;

pub use error::AbstractInterfaceError;

pub use crate::{deployers::*, deployment::*};
