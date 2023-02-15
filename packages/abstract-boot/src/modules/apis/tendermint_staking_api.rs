use abstract_os::{api::InstantiateMsg, tendermint_staking::*};
use boot_core::{prelude::boot_contract, BootEnvironment, Contract};
use cosmwasm_std::Empty;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct TMintStakingApi<Chain>;

impl<Chain: BootEnvironment> TMintStakingApi<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
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
