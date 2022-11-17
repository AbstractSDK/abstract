//! # Abstract extension
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};
pub type ExtensionResult<C = Empty> = Result<Response<C>, ExtensionError>;
// Default to Empty

pub use crate::state::ExtensionContract;
pub use error::ExtensionError;

pub mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
mod handler;
mod schema;
pub mod state;
