pub const KUJIRA: &str = "kujira";
#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::std::registry::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::std::registry::KUJIRA;

pub mod dex;
pub mod money_market;
pub mod staking;
