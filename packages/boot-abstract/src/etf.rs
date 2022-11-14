use crate::AbstractOS;
use abstract_os::add_on::MigrateMsg;
use abstract_os::etf::*;
use boot_core::{Contract, IndexResponse, TxHandler, TxResponse};

pub type ETF<Chain> = AbstractOS<Chain, EtfExecuteMsg, EtfInstantiateMsg, EtfQueryMsg, MigrateMsg>;

impl<Chain: TxHandler + Clone> ETF<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
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
