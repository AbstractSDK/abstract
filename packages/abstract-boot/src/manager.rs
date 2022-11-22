use std::fmt::Debug;

use abstract_sdk::os::objects::module::{ModuleInfo, ModuleVersion};
use boot_core::state::StateInterface;
use cosmwasm_std::{to_binary, Addr, Binary};

use serde::Serialize;

use abstract_sdk::os::manager::*;

use boot_core::{BootEnvironment, BootError, Contract, IndexResponse, TxResponse};

use boot_core::interface::BootExecute;
use boot_core::interface::ContractInstance;
use boot_core::prelude::boot_contract;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Manager<Chain>;

impl<Chain: BootEnvironment> Manager<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("manager"),
            // .with_mock(Box::new(
            //     ContractWrapper::new_with_empty(
            //         ::contract::execute,
            //         ::contract::instantiate,
            //         ::contract::query,
            //     ),
            // ))
        )
    }

    // pub fn update_terraswap_trader(
    //     &self,
    //     extension: &str,
    //     to_add: Option<Vec<String>>,
    //     to_remove: Option<Vec<String>>,
    // ) -> Result<(), BootError> {
    //     self.execute(
    //         &ExecuteMsg::ExecOnModule {
    //             module_id: extension.into(),
    //             exec_msg: to_binary(&<ExtensionExecuteMsg<Empty>>::Configure(BaseExecuteMsg::UpdateTraders {
    //                 to_add,
    //                 to_remove,
    //             }))
    //             .unwrap(),
    //         },
    //         None,
    //     )?;
    //     Ok(())
    // }

    pub fn upgrade_module<M: Serialize>(
        &self,
        module_id: &str,
        migrate_msg: &M,
    ) -> Result<(), BootError> {
        self.execute(
            &ExecuteMsg::Upgrade {
                module: ModuleInfo::from_id(module_id, ModuleVersion::Latest {})?,
                migrate_msg: Some(to_binary(migrate_msg)?),
            },
            None,
        )?;
        Ok(())
    }

    pub fn install_module<M: Serialize>(
        &self,
        module_id: &str,
        init_msg: Option<&M>,
    ) -> Result<(), BootError> {
        self.execute(
            &ExecuteMsg::InstallModule {
                module: ModuleInfo::from_id(module_id, ModuleVersion::Latest {})?,
                init_msg: init_msg.map(to_binary).transpose()?,
            },
            None,
        )?;
        Ok(())
    }

    pub fn execute_on_module(&self, module: &str, msg: impl Serialize) -> Result<(), BootError> {
        self.execute(
            &ExecuteMsg::ExecOnModule {
                module_id: module.into(),
                exec_msg: to_binary(&msg).unwrap(),
            },
            None,
        )?;
        Ok(())
    }

    pub fn add_module<
        I: Serialize + Debug,
        H: Serialize + Debug,
        N: Serialize + Debug,
        S: Serialize + Debug,
    >(
        &self,
        module: &Contract<Chain>,
        init_msg: Option<&I>,
        contract_id: &str,
        version: String,
    ) -> Result<TxResponse<Chain>, BootError> {
        let mut msg: Option<Binary> = None;
        if init_msg.is_some() {
            msg = init_msg.map(|msg| to_binary(msg).unwrap());
        }
        let result = self.execute(
            &ExecuteMsg::InstallModule {
                module: ModuleInfo::from_id(contract_id, ModuleVersion::Version(version))?,
                init_msg: msg,
            },
            None,
        )?;

        let module_address = result.event_attr_value("wasm", "new module:")?;
        self.get_chain()
            .state()
            .set_address(&module.id, &Addr::unchecked(module_address));

        Ok(result)
    }
}

// pub fn get_module_kind(name: &str) -> anyhow::Result<ModuleKind> {
//     if [TERRASWAP].contains(&name) {
//         return Ok(ModuleKind::Extension);
//     } else if [LIQUIDITY_INTERFACE, SUBSCRIPTION].contains(&name) {
//         Ok(ModuleKind::App)
//     } else {
//         return Err(anyhow::Error::msg(
//             "The requested module to be added is not a module",
//         ));
//     }
// }
