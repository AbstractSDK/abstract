// Wyndex is only available on juno
pub const WYNDEX: &str = "wyndex";

#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::LOCAL;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::JUNO;

pub mod dex;
pub mod staking;
