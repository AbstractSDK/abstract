pub const ASTROPORT: &str = "astroport";
#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::core::registry::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
lazy_static::lazy_static! {
    pub static ref AVAILABLE_CHAINS: Vec<&'static str> = {
        let mut v = Vec::new();
        v.extend_from_slice(abstract_sdk::core::registry::NEUTRON);
        v.extend_from_slice(abstract_sdk::core::registry::TERRA);
        v.extend_from_slice(abstract_sdk::core::registry::SEI);
        v.extend_from_slice(abstract_sdk::core::registry::INJECTIVE);
        v
    };
}

pub mod dex;
pub mod staking;
