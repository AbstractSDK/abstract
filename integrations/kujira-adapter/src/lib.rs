pub const KUJIRA: &str = "kujira";
#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::framework::registry::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::framework::registry::KUJIRA;

pub mod dex;
pub mod staking;
