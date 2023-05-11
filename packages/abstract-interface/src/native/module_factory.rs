use abstract_core::module_factory::*;

use cw_orch::{environment::TxHandler, interface, prelude::*};

pub use abstract_core::module_factory::{
    ExecuteMsgFns as MFactoryExecFns, QueryMsgFns as MFactoryQueryFns,
};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct ModuleFactory<Chain>;

impl<Chain: CwEnv> Uploadable for ModuleFactory<Chain> {
    #[cfg(feature = "integration")]
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
        ArtifactsDir::env()
            .find_wasm_path("module_factory")
            .unwrap()
    }
}

impl<Chain: CwEnv> ModuleFactory<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(cw_orch::contract::Contract::new(name, chain))
    }

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

    // pub  fn save_init_binaries(&self, mem_addr: String, version_control_addr: String) -> Result<(), crate::AbstractBootError> {
    //     let msgs = get_adapter_init_msgs(mem_addr,version_control_addr);
    //     // TODO: Add version management support
    //     let binaries = msgs
    //         .iter()
    //         .map(|(name, msg)| ((name.clone(), "v0.1.0".to_string()), msg.clone()))
    //         .collect::<Vec<_>>();
    //     self.0
    //         .execute(
    //             &ExecuteMsg::UpdateFactoryBinaryMsgs {
    //                 to_add: binaries,
    //                 to_remove: vec![(LIQUIDITY_INTERFACE.to_string(), "v0.1.0".to_string())],
    //             },
    //             &vec![],
    //         )
    //         ?;
    //     Ok(())
    // }
}
