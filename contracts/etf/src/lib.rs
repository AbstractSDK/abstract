pub mod contract;
pub mod error;
mod handlers;
pub mod msg;
pub mod response;
pub mod state;

pub const ETF: &str = "abstract:etf";
// TODO; FIX
// #[cfg(test)]
// #[cfg(not(target_arch = "wasm32"))]
// mod tests;
#[cfg(feature = "boot")]
pub mod boot {
    use abstract_os::{app::MigrateMsg, etf::*};
    use boot_core::{prelude::boot_contract, BootEnvironment, Contract};

    #[boot_contract(EtfInstantiateMsg, EtfExecuteMsg, EtfQueryMsg, MigrateMsg)]
    pub struct ETF<Chain>;

    impl<Chain: BootEnvironment> ETF<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            let mut contract = Contract::new(name, chain);
            contract = contract.with_wasm_path("etf");
            Self(contract)
        }
    }
}
