pub(crate) mod commands;
pub mod contract;
pub(crate) mod dex_trait;
pub mod error;
mod exchanges;
pub(crate) mod queries;

pub use dex_trait::DEX;

// TODO: FIX
// #[cfg(test)]
// #[cfg(not(target_arch = "wasm32"))]
// mod tests;
