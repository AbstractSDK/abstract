use abstract_os::extension::InstantiateMsg;
use boot_core::{prelude::boot_contract, BootEnvironment, Contract};
use cosmwasm_std::Empty;

use abstract_os::tendermint_staking::*;

#[boot_contract(
    InstantiateMsg,
    TendermintStakingExecuteMsg,
    TendermintStakingQueryMsg,
    Empty
)]
pub struct TMintStakingExtension<Chain>;

impl<Chain: BootEnvironment> TMintStakingExtension<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("tendermint_staking"),
            // .with_mock(Box::new(
            //     ContractWrapper::new_with_empty(
            //         ::contract::execute,
            //         ::contract::instantiate,
            //         ::contract::query,
            //     ),
            // ))
        )
    }
}
