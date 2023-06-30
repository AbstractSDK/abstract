pub use abstract_core::manager::{ExecuteMsgFns as ManagerExecFns, QueryMsgFns as ManagerQueryFns};
use abstract_core::{
    adapter,
    manager::*,
    objects::module::{ModuleInfo, ModuleVersion},
};
use cosmwasm_std::{to_binary, Empty};
use cw_orch::environment::TxHandler;
use cw_orch::interface;
use cw_orch::prelude::*;
use serde::Serialize;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Manager<Chain>;

impl<Chain: CwEnv> Uploadable for Manager<Chain> {
    fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::manager::contract::execute,
                ::manager::contract::instantiate,
                ::manager::contract::query,
            )
            .with_migrate(::manager::contract::migrate),
        )
    }
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("manager")
            .unwrap()
    }
}

impl<Chain: CwEnv> Manager<Chain> {
    pub fn upgrade_module<M: Serialize>(
        &self,
        module_id: &str,
        migrate_msg: &M,
    ) -> Result<(), crate::AbstractInterfaceError> {
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

    pub fn replace_api(
        &self,
        module_id: &str,
        funds: Option<&[Coin]>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        // this should check if installed?
        self.uninstall_module(module_id.to_string())?;

        self.install_module(module_id, &Empty {}, funds)
    }

    pub fn install_module<TInitMsg: Serialize>(
        &self,
        module_id: &str,
        init_msg: &TInitMsg,
        funds: Option<&[Coin]>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        self.install_module_version(module_id, ModuleVersion::Latest, init_msg, funds)
    }

    pub fn install_module_version<M: Serialize>(
        &self,
        module_id: &str,
        version: ModuleVersion,
        init_msg: &M,
        funds: Option<&[Coin]>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        self.execute(
            &ExecuteMsg::InstallModule {
                module: ModuleInfo::from_id(module_id, version)?,
                init_msg: Some(to_binary(init_msg).unwrap()),
            },
            funds,
        )?;
        Ok(())
    }

    pub fn execute_on_module(
        &self,
        module: &str,
        msg: impl Serialize,
    ) -> Result<<Chain as cw_orch::prelude::TxHandler>::Response, crate::AbstractInterfaceError>
    {
        self.execute(
            &ExecuteMsg::ExecOnModule {
                module_id: module.into(),
                exec_msg: to_binary(&msg).unwrap(),
            },
            None,
        )
        .map_err(Into::into)
    }

    pub fn update_adapter_authorized_addresses(
        &self,
        module_id: &str,
        to_add: Vec<String>,
        to_remove: Vec<String>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        self.execute_on_module(
            module_id,
            adapter::ExecuteMsg::<Empty, Empty>::Base(
                adapter::BaseExecuteMsg::UpdateAuthorizedAddresses { to_add, to_remove },
            ),
        )?;

        Ok(())
    }

    /// Return the module info installed on the manager
    pub fn module_info(
        &self,
        module_id: &str,
    ) -> Result<Option<ManagerModuleInfo>, crate::AbstractInterfaceError> {
        let module_infos = self.module_infos(None, None)?.module_infos;
        let found = module_infos
            .into_iter()
            .find(|module_info| module_info.id == module_id);
        Ok(found)
    }

    pub fn is_module_installed(
        &self,
        module_id: &str,
    ) -> Result<bool, crate::AbstractInterfaceError> {
        let module = self.module_info(module_id)?;
        Ok(module.is_some())
    }
}
