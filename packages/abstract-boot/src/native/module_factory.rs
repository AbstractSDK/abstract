use abstract_os::module_factory::*;

// use crate::api::get_api_init_msgs;
use boot_core::{BootEnvironment, Contract, TxResponse};

pub use abstract_os::module_factory::{
    ExecuteMsgFns as MFactoryExecFns, QueryMsgFns as MFactoryQueryFns,
};
use boot_core::{boot_contract, BootExecute};

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct ModuleFactory<Chain>;

impl<Chain: BootEnvironment> ModuleFactory<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("module_factory");
        Self(contract)
    }

    pub fn change_ans_host_addr(
        &self,
        mem_addr: String,
    ) -> Result<TxResponse<Chain>, crate::AbstractBootError> {
        self.execute(
            &ExecuteMsg::UpdateConfig {
                admin: None,
                ans_host_address: Some(mem_addr),
                version_control_address: None,
            },
            None,
        )
        .map_err(Into::into)
    }

    // pub  fn save_init_binaries(&self, mem_addr: String, version_control_addr: String) -> Result<(), crate::AbstractBootError> {
    //     let msgs = get_api_init_msgs(mem_addr,version_control_addr);
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
