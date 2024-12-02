#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::std::constants::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::std::constants::KUJIRA;

pub mod dex;
pub mod money_market;
pub mod staking;
