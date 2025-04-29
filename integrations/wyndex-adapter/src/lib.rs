// Wyndex is only available on juno
pub const WYNDEX: &str = "wyndex";

#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::std::constants::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::std::constants::JUNO;

pub mod dex;
pub mod staking;
