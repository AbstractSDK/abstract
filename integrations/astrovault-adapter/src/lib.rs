pub const ASTROVAULT: &str = "astrovault";
#[cfg(feature = "local")]
pub const AVAILABLE_CHAINS: &[&str] = abstract_sdk::std::registry::LOCAL_CHAIN;
#[cfg(not(feature = "local"))]
lazy_static::lazy_static! {
    pub static ref AVAILABLE_CHAINS: Vec<&'static str> = {
        let mut v = Vec::new();
        v.extend_from_slice(abstract_sdk::std::registry::ARCHWAY);
        v
    };
}

pub mod dex;
pub mod staking;

#[cfg(feature = "full_integration")]
pub mod mini_astrovault;
