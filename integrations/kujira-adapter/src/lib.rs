pub const KUJIRA: &str = "kujira";
#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::KUJIRA;

pub mod dex;
pub mod money_market;
pub mod staking;
