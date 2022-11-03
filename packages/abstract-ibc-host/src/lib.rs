//! # Abstract API
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};

pub use error::HostError;

pub use crate::state::Host;
pub mod chains;
pub mod error;
mod execute;
pub(crate) mod host_commands;
pub mod instantiate;
pub mod os_commands;
mod query;
mod reply;
pub mod state;
/// Abstract SDK trait implementations
pub mod traits;

// Default to Empty
pub type ApiResult<C = Empty> = Result<Response<C>, HostError>;
