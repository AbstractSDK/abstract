//! # Abstract api
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};

pub type ApiResult<C = Empty> = Result<Response<C>, ApiError>;
// Default to Empty

pub use crate::state::ApiContract;
pub use error::ApiError;

pub mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
mod handler;
pub mod state;

#[cfg(feature = "schema")]
mod schema;
