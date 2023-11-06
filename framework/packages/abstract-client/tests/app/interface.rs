use cw_orch::{interface, prelude::*};

use super::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct CounterContract;

impl<Chain: CwEnv> Uploadable for CounterContract<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("counter")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                super::contract::execute,
                super::contract::instantiate,
                super::contract::query,
            )
            .with_migrate(super::contract::migrate),
        )
    }
}
