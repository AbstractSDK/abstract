use abstract_sdk::os::{app::MigrateMsg, etf::*};
use boot_core::{prelude::boot_contract, BootEnvironment, Contract};

#[boot_contract(EtfInstantiateMsg, EtfExecuteMsg, EtfQueryMsg, MigrateMsg)]
pub struct ETF<Chain>;

impl<Chain: BootEnvironment> ETF<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("etf"), // .with_mock(Box::new(
                                                              //     ContractWrapper::new_with_empty(
                                                              //         ::contract::execute,
                                                              //         ::contract::instantiate,
                                                              //         ::contract::query,
                                                              //     ),
                                                              // ))
        )
    }
}
