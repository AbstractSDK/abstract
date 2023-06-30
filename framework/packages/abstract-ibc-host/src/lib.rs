//! # Abstract Adapter
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};

pub use error::HostError;

pub use crate::state::Host;
pub mod account_commands;
pub mod chains;
pub mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
mod handler;
pub(crate) mod host_commands;
#[cfg(feature = "schema")]
mod schema;
pub mod state;

// Default to Empty
pub type AdapterResult<C = Empty> = Result<Response<C>, HostError>;
