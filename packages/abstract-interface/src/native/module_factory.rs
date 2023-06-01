use abstract_core::module_factory::*;

use cw_orch::{environment::TxHandler, interface, prelude::*};

pub use abstract_core::module_factory::{
    ExecuteMsgFns as MFactoryExecFns, QueryMsgFns as MFactoryQueryFns,
};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct ModuleFactory<Chain>;

impl<Chain: CwEnv> Uploadable for ModuleFactory<Chain> {
    fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::module_factory::contract::execute,
                ::module_factory::contract::instantiate,
                ::module_factory::contract::query,
            )
            .with_migrate(::module_factory::contract::migrate)
            .with_reply(::module_factory::contract::reply),
        )
    }
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("module_factory")
            .unwrap()
    }
}

impl<Chain: CwEnv> ModuleFactory<Chain> {
    pub fn change_ans_host_addr(
        &self,
        mem_addr: String,
    ) -> Result<TxResponse<Chain>, crate::AbstractInterfaceError> {
        self.execute(
            &ExecuteMsg::UpdateConfig {
                ans_host_address: Some(mem_addr),
                version_control_address: None,
            },
            None,
        )
        .map_err(Into::into)
    }
}
