pub const ASTROPORT: &str = "astroport";
#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::TERRA;

pub mod dex;
pub mod staking;
