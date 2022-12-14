use boot_core::{prelude::boot_contract, BootEnvironment, Contract};
use cosmwasm_std::Empty;

use abstract_sdk::os::{extension, tendermint_staking, tendermint_staking::*};

type TMintStakingInstantiateMsg = extension::InstantiateMsg;
type TMintStakingExecuteMsg = extension::ExecuteMsg<RequestMsg>;
type TMintStakingQueryMsg = extension::QueryMsg<tendermint_staking::QueryMsg>;

#[boot_contract(
    TMintStakingInstantiateMsg,
    TMintStakingExecuteMsg,
    TMintStakingQueryMsg,
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
