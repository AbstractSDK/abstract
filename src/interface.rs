use crate::msg::*;
use abstract_boot::AppDeployer;
use abstract_core::app::MigrateMsg;
use cosmwasm_std::Empty;
use cw_orch::{ContractWrapper, Mock, MockContract, TxHandler, Uploadable, WasmPath};
use cw_orch::{contract, Contract, CwEnv};

#[contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Template<Chain>;

impl<Chain: CwEnv> AppDeployer<Chain> for Template<Chain> {}

impl<Chain: CwEnv> Template<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}

impl Uploadable for Template<Mock> {
    fn wasm(&self) -> WasmPath {
        WasmPath::new("template_app").unwrap()
    }

    fn wrapper(&self) -> Box<dyn MockContract<Empty, Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
                .with_migrate(crate::contract::migrate),
        )
    }
}
