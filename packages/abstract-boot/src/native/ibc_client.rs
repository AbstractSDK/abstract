use boot_core::{BootEnvironment, Contract};

use abstract_sdk::os::ibc_client::*;

use boot_core::prelude::boot_contract;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct IbcClient<Chain>;

impl<Chain: BootEnvironment> IbcClient<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("ibc_client"), // .with_mock(Box::new(
                                                                     //     ContractWrapper::new_with_empty(
                                                                     //         ::contract::execute,
                                                                     //         ::contract::instantiate,
                                                                     //         ::contract::query,
                                                                     //     ),
                                                                     // ))
        )
    }
}
