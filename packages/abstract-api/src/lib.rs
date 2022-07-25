//! # Abstract API
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};

pub use error::ApiError;

pub use crate::state::ApiContract;

pub mod error;
mod execute;
pub mod instantiate;
mod query;
pub mod state;

// Default to Empty
pub type ApiResult<C = Empty> = Result<Response<C>, ApiError>;
