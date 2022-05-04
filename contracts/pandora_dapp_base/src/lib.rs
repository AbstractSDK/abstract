// #[cfg(test)]
// mod mock_querier;
use cosmwasm_std::{Empty, Response};

pub use error::DappError;

pub use crate::state::DappContract;

pub mod error;
mod execute;
pub mod instantiate;
mod query;
pub mod state;

// #[cfg(test)]
// mod testing;

// This is a simple type to let us handle future extensions
pub type Extension = Option<Empty>;
// Default to Empty
pub type DappResult<C = Empty> = Result<Response<C>, DappError>;
