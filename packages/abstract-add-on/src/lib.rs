// #[cfg(test)]
// mod mock_querier;
use cosmwasm_std::{Empty, Response};

pub use error::AddOnError;

pub use crate::state::AddOnContract;

pub mod error;
mod execute;
mod ibc_callback;
pub mod instantiate;
mod query;
mod receive;
pub mod state;
/// Abstract SDK trait implementations
pub mod traits;
// #[cfg(test)]
// mod testing;

// Default to Empty
pub type AddOnResult<C = Empty> = Result<Response<C>, AddOnError>;
