pub(crate) mod commands;
pub mod contract;
pub(crate) mod dex_trait;
pub mod error;
mod exchanges;

pub(crate) mod handlers;

pub use commands::LocalDex;
pub use dex_trait::DEX;

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_exchange {
    pub use super::exchanges::osmosis::Osmosis;
}

// TODO: FIX
// #[cfg(test)]
// #[cfg(not(target_arch = "wasm32"))]
// mod tests;
