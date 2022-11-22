pub use abstract_sdk::os::balancer::*;
use boot_core::{Contract, IndexResponse, TxHandler, TxResponse, BootEnvironment};
use cosmwasm_std::Empty;

use crate::AbstractOS;

use boot_core::prelude::boot_contract;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct Balancer<Chain>;

impl<Chain: BootEnvironment> Balancer<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("balancer"), // .with_mock(Box::new(
                                                                   //     ContractWrapper::new_with_empty(
                                                                   //         ::contract::execute,
                                                                   //         ::contract::instantiate,
                                                                   //         ::contract::query,
                                                                   //     ),
                                                                   // ))
        )
    }
}
