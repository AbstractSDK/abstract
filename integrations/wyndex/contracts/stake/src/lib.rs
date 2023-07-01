//#![warn(missing_docs)]
#![doc(html_logo_url = "../../../uml/logo.png")]
//! # WYNDEX Staking
//!
//! ## Description
//!
//! We need a project that allow LP tokens to be staked.
//!
//! ## Objectives
//!
//! The main goal of the **WYNDEX staking** is to:
//!   - Allow the LP TOKEN to be staked with a proper curve and time.
//!

/// Main contract logic
pub mod contract;
/// Lazy reward distribution, mostly can be reused by other contracts
pub mod distribution;

/// custom error handler
mod error;

/// custom input output messages
pub mod msg;

/// state on the blockchain
pub mod state;

#[cfg(test)]
mod multitest;
/// some helper functions
mod utils;
pub use crate::error::ContractError;
