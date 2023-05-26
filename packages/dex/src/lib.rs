pub mod adapter;
mod api;
pub mod command;
pub mod error;
pub mod msg;
pub mod state;

// Export interface for use in SDK modules
pub use adapter::DexAdapter;
pub use api::{Dex, DexInterface};

pub const EXCHANGE: &str = "abstract:dex";
