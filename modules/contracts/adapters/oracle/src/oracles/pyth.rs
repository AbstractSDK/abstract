use abstract_adapter_utils::Identify;

pub const PYTH: &str = "pyth";

#[derive(Default)]
pub struct Pyth {}

impl Identify for Pyth {
    fn name(&self) -> &'static str {
        PYTH
    }

    fn is_available_on(&self, chain_name: &str) -> bool {
        chain_name == "pyth"
    }
}

#[cfg(feature = "pyth")]
mod integration {}
