pub mod contract;
mod error;
pub mod msg;
pub mod state;

#[cfg(test)]
mod multitest;

pub use error::ContractError;

// #[cfg(test)]
// mod multitest;
