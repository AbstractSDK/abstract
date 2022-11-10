use abstract_os::module_factory::*;

// use crate::api::get_api_init_msgs;
use crate::AbstractOS;
use boot_core::{BootError, Contract, IndexResponse, TxHandler, TxResponse};

pub type ModuleFactory<Chain> = AbstractOS<Chain, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg>;

impl<Chain: TxHandler + Clone> ModuleFactory<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("module_factory"), // .with_mock(Box::new(
                                                                         //     ContractWrapper::new_with_empty(
                                                                         //         ::contract::execute,
                                                                         //         ::contract::instantiate,
                                                                         //         ::contract::query,
                                                                         //     ),
                                                                         // ))
        )
    }

    pub fn change_memory_addr(&self, mem_addr: String) -> Result<TxResponse<Chain>, BootError> {
        self.execute(
            &ExecuteMsg::UpdateConfig {
                admin: None,
                memory_address: Some(mem_addr),
                version_control_address: None,
            },
            None,
        )
    }

    // pub  fn save_init_binaries(&self, mem_addr: String, version_control_addr: String) -> Result<(), BootError> {
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
