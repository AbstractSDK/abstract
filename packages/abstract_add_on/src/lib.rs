// #[cfg(test)]
// mod mock_querier;
use cosmwasm_std::{Empty, Response};

pub use error::AddOnError;

pub use crate::state::AddOnContract;

pub mod error;
mod execute;
pub mod instantiate;
mod query;
pub mod state;

// #[cfg(test)]
// mod testing;

// Default to Empty
pub type AddOnResult<C = Empty> = Result<Response<C>, AddOnError>;
