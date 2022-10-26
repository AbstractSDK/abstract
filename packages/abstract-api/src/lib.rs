#![feature(generic_associated_types)]
//! # Abstract API
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};
pub type ApiResult<C = Empty> = Result<Response<C>, ApiError>;
// Default to Empty

pub use crate::state::ApiContract;
pub use error::ApiError;

pub mod error;
mod execute;
mod ibc_callback;
pub mod instantiate;
mod query;
mod receive;
pub mod state;
/// Abstract SDK trait implementations
pub mod traits;
