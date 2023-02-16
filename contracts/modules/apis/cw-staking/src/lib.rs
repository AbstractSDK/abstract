pub mod contract;
pub mod error;
mod providers;

mod handlers;
mod traits;

pub use traits::cw_staking_adapter::CwStakingAdapter;
pub use traits::local_cw_staking::LocalCwStaking;

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub mod host_staking {
    pub use super::providers::osmosis::Osmosis;
}

// #[cfg(test)]
// #[cfg(not(target_arch = "wasm32"))]
// mod tests;
