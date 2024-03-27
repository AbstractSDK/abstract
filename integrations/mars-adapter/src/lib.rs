pub const MARS: &str = "mars";
#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
pub const AVAILABLE_CHAINS: &[&str] = &["pion", "neutron", "osmosis", "osmo", "osmo-test"];
pub mod money_market;
