use abstract_sdk::os::extension::*;

use abstract_sdk::os::{tendermint_staking::*};
use cosmwasm_std::Empty;

use crate::AbstractOS;
use boot_core::{Contract, IndexResponse, TxHandler, TxResponse};

pub type TMintStakingExtension<Chain> = AbstractOS<
    Chain,
    ExecuteMsg<RequestMsg>,
    abstract_sdk::os::extension::InstantiateMsg,
    abstract_sdk::os::extension::QueryMsg<abstract_sdk::os::tendermint_staking::QueryMsg>,
    Empty,
>;

impl<Chain: TxHandler + Clone> TMintStakingExtension<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
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
