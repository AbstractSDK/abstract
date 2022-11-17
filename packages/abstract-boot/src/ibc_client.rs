use boot_core::{Contract, IndexResponse, TxHandler, TxResponse};

use crate::AbstractOS;
use abstract_sdk::os::ibc_client::*;

pub type IbcClient<Chain> = AbstractOS<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg>;

impl<Chain: TxHandler + Clone> IbcClient<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
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
