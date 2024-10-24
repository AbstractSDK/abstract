use abstract_std::module_factory::*;
pub use abstract_std::module_factory::{
    ExecuteMsgFns as MFactoryExecFns, QueryMsgFns as MFactoryQueryFns,
};
use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct ModuleFactory<Chain>;

impl<Chain: CwEnv> cw_blob::interface::DeterministicInstantiation<Chain> for ModuleFactory<Chain> {}

impl<Chain: CwEnv> Uploadable for ModuleFactory<Chain> {
    fn wrapper() -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::module_factory::contract::execute,
                ::module_factory::contract::instantiate,
                ::module_factory::contract::query,
            )
            .with_migrate(::module_factory::contract::migrate),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        let build_postfix = {
            #[cfg(feature = "mock-deployment")]
            {
                cw_orch::build::BuildPostfix::Custom("mock".to_string())
            }
            #[cfg(not(feature = "mock-deployment"))]
            {
                cw_orch::build::BuildPostfix::None
            }
        };
        artifacts_dir_from_workspace!()
            .find_wasm_path_with_build_postfix("module_factory", build_postfix)
            .unwrap()
    }
}
