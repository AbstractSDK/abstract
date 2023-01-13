use abstract_os::objects::module::{ModuleInfo, ModuleVersion};
use boot_core::BootEnvironment;
use cosmwasm_std::{to_binary, Empty};
use serde::Serialize;

use abstract_os::manager::*;
pub use abstract_os::manager::{ExecuteMsgFns as ManagerExecFns, QueryMsgFns as ManagerQueryFns};

use boot_core::{BootError, Contract};

use boot_core::{interface::BootExecute, prelude::boot_contract};

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Manager<Chain>;

impl<Chain: BootEnvironment> Manager<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        let mut contract = Contract::new(name, chain);
        contract = contract.with_wasm_path("manager");
        Self(contract)
    }

    // pub fn update_terraswap_trader(
    //     &self,
    //     api: &str,
    //     to_add: Option<Vec<String>>,
    //     to_remove: Option<Vec<String>>,
    // ) -> Result<(), BootError> {
    //     self.execute(
    //         &ExecuteMsg::ExecOnModule {
    //             module_id: api.into(),
    //             exec_msg: to_binary(&<ApiExecuteMsg<Empty>>::Configure(BaseExecuteMsg::UpdateTraders {
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
                modules: vec![(
                    ModuleInfo::from_id(module_id, ModuleVersion::Latest)?,
                    Some(to_binary(migrate_msg)?),
                )],
            },
            None,
        )?;
        Ok(())
    }

    pub fn replace_api(&self, module_id: &str) -> Result<(), BootError> {
        // this should check if installed?
        self.uninstall_module(module_id)?;

        self.install_module(module_id, &Empty {})
    }

    pub fn install_module<TInitMsg: Serialize>(
        &self,
        module_id: &str,
        init_msg: &TInitMsg,
    ) -> Result<(), BootError> {
        self.install_module_version(module_id, ModuleVersion::Latest, init_msg)
    }

    pub fn install_module_version<M: Serialize>(
        &self,
        module_id: &str,
        version: ModuleVersion,
        // not option
        init_msg: &M,
    ) -> Result<(), BootError> {
        self.execute(
            &ExecuteMsg::InstallModule {
                module: ModuleInfo::from_id(module_id, version)?,
                init_msg: Some(to_binary(init_msg)?),
            },
            None,
        )?;
        Ok(())
    }

    pub fn uninstall_module(&self, module_id: impl Into<String>) -> Result<(), BootError> {
        self.execute(
            &ExecuteMsg::RemoveModule {
                module_id: module_id.into(),
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
}

// pub fn get_module_kind(name: &str) -> anyhow::Result<ModuleKind> {
//     if [TERRASWAP].contains(&name) {
//         return Ok(ModuleKind::Api);
//     } else if [LIQUIDITY_INTERFACE, SUBSCRIPTION].contains(&name) {
//         Ok(ModuleKind::App)
//     } else {
//         return Err(anyhow::Error::msg(
//             "The requested module to be added is not a module",
//         ));
//     }
// }
