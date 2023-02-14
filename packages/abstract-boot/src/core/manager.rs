use abstract_os::api;
use abstract_os::manager::*;
pub use abstract_os::manager::{ExecuteMsgFns as ManagerExecFns, QueryMsgFns as ManagerQueryFns};
use abstract_os::objects::module::{ModuleInfo, ModuleVersion};
use boot_core::BootEnvironment;
use boot_core::{interface::BootExecute, prelude::boot_contract};
use boot_core::{Contract};
use cosmwasm_std::{to_binary, Empty};
use serde::Serialize;

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
    // ) -> Result<(), crate::AbstractBootError> {
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
    ) -> Result<(), crate::AbstractBootError> {
        self.execute(
            &ExecuteMsg::Upgrade {
                modules: vec![(
                    ModuleInfo::from_id(module_id, ModuleVersion::Latest)?,
                    Some(to_binary(migrate_msg).unwrap()),
                )],
            },
            None,
        )?;
        Ok(())
    }

    pub fn replace_api(&self, module_id: &str) -> Result<(), crate::AbstractBootError> {
        // this should check if installed?
        self.uninstall_module(module_id)?;

        self.install_module(module_id, &Empty {})
    }

    pub fn install_module<TInitMsg: Serialize>(
        &self,
        module_id: &str,
        init_msg: &TInitMsg,
    ) -> Result<(), crate::AbstractBootError> {
        self.install_module_version(module_id, ModuleVersion::Latest, init_msg)
    }

    pub fn install_module_version<M: Serialize>(
        &self,
        module_id: &str,
        version: ModuleVersion,
        init_msg: &M,
    ) -> Result<(), crate::AbstractBootError> {
        self.execute(
            &ExecuteMsg::InstallModule {
                module: ModuleInfo::from_id(module_id, version)?,
                init_msg: Some(to_binary(init_msg).unwrap()),
            },
            None,
        )?;
        Ok(())
    }

    pub fn uninstall_module(
        &self,
        module_id: impl Into<String>,
    ) -> Result<(), crate::AbstractBootError> {
        self.execute(
            &ExecuteMsg::RemoveModule {
                module_id: module_id.into(),
            },
            None,
        )?;
        Ok(())
    }

    pub fn execute_on_module(
        &self,
        module: &str,
        msg: impl Serialize,
    ) -> Result<(), crate::AbstractBootError> {
        self.execute(
            &ExecuteMsg::ExecOnModule {
                module_id: module.into(),
                exec_msg: to_binary(&msg).unwrap(),
            },
            None,
        )?;
        Ok(())
    }

    pub fn update_api_traders(
        &self,
        module_id: &str,
        to_add: Vec<String>,
        to_remove: Vec<String>,
    ) -> Result<(), crate::AbstractBootError> {
        self.execute_on_module(
            module_id,
            api::ExecuteMsg::<Empty, Empty>::Base(api::BaseExecuteMsg::UpdateTraders {
                to_add,
                to_remove,
            }),
        )?;

        Ok(())
    }

    /// Return the module info installed on the manager
    pub fn module_info(
        &self,
        module_id: &str,
    ) -> Result<Option<ManagerModuleInfo>, crate::AbstractBootError> {
        let module_infos = self.module_infos(None, None)?.module_infos;
        let found = module_infos
            .into_iter()
            .find(|module_info| module_info.id == module_id);
        Ok(found)
    }

    pub fn is_module_installed(&self, module_id: &str) -> Result<bool, crate::AbstractBootError> {
        let module = self.module_info(module_id)?;
        Ok(module.is_some())
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
