// Wyndex is only available on juno
pub const WYNDEX: &str = "wyndex";

#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::framework::registry::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::framework::registry::JUNO;

pub mod dex;
pub mod staking;
