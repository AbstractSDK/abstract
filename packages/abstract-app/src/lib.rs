// #[cfg(test)]
// mod mock_querier;
pub use crate::state::AppContract;
pub(crate) use abstract_sdk::base::*;
use cosmwasm_std::{Empty, Response};
pub use error::AppError;

mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
pub(crate) mod handler;
#[cfg(feature = "schema")]
mod schema;
pub mod state;
// #[cfg(test)]
// mod testing;
// Default to Empty
pub type AppResult<C = Empty> = Result<Response<C>, AppError>;
