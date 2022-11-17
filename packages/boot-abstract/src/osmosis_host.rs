use crate::AbstractOS;
use abstract_sdk::os::ibc_host::*;
use boot_core::{Contract, IndexResponse, TxHandler, TxResponse};
use cosmwasm_std::Empty;

pub type OsmosisHost<Chain> = AbstractOS<Chain, Empty, BaseInstantiateMsg, QueryMsg, MigrateMsg>;

impl<Chain: TxHandler + Clone> OsmosisHost<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("osmosis_host"), // .with_mock(Box::new(
                                                                       //     ContractWrapper::new_with_empty(
                                                                       //         ::contract::execute,
                                                                       //         ::contract::instantiate,
                                                                       //         ::contract::query,
                                                                       //     ),
                                                                       // ))
        )
    }
}
