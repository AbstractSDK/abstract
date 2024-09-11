pub use abstract_std::account::{ExecuteMsgFns as ManagerExecFns, QueryMsgFns as ManagerQueryFns};
use abstract_std::{
    account::responses::AccountModuleInfo,
    account::*,
    adapter::{self, AdapterBaseMsg},
    ibc_host::{HelperAction, HostAction},
    module_factory::SimulateInstallModulesResponse,
    objects::{
        module::{ModuleInfo, ModuleVersion},
        AccountId, TruncatedChainId,
    },
    ACCOUNT, IBC_CLIENT,
};
use cosmwasm_std::{to_json_binary, Binary, Empty};
use cw_orch::{environment::Environment, interface, prelude::*};
use serde::Serialize;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Account<Chain>;

impl<Chain: CwEnv> Account<Chain> {
    pub(crate) fn new_from_id(account_id: &AccountId, chain: Chain) -> Self {
        let manager_id = format!("{ACCOUNT}-{account_id}");
        Self::new(manager_id, chain)
    }
}

impl<Chain: CwEnv> Uploadable for Account<Chain> {
    fn wrapper() -> <Mock as TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::account::contract::execute,
                ::account::contract::instantiate,
                ::account::contract::query,
            )
            .with_migrate(::account::contract::migrate)
            .with_reply(::account::contract::reply),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("manager")
            .unwrap()
    }
}

impl<Chain: CwEnv> Account<Chain> {
    pub fn upgrade_module<M: Serialize>(
        &self,
        module_id: &str,
        migrate_msg: &M,
    ) -> Result<(), crate::AbstractInterfaceError> {
        self.execute(
            &ExecuteMsg::Upgrade {
                modules: vec![(
                    ModuleInfo::from_id(module_id, ModuleVersion::Latest)?,
                    Some(to_json_binary(migrate_msg).unwrap()),
                )],
            },
            &[],
        )?;
        Ok(())
    }

    pub fn replace_api(
        &self,
        module_id: &str,
        funds: &[Coin],
    ) -> Result<(), crate::AbstractInterfaceError> {
        // this should check if installed?
        self.uninstall_module(module_id.to_string())?;

        self.install_module::<Empty>(module_id, None, funds)?;
        Ok(())
    }

    pub fn install_modules(
        &self,
        modules: Vec<ModuleInstallConfig>,
        funds: &[Coin],
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        self.execute(&ExecuteMsg::InstallModules { modules }, funds)
            .map_err(Into::into)
    }

    pub fn install_modules_auto(
        &self,
        modules: Vec<ModuleInstallConfig>,
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        let config = self.config()?;
        let module_infos = modules.iter().map(|m| m.module.clone()).collect();
        let sim_response: SimulateInstallModulesResponse = self
            .environment()
            .query(
                &abstract_std::module_factory::QueryMsg::SimulateInstallModules {
                    modules: module_infos,
                },
                &config.module_factory_address,
            )
            .map_err(Into::into)?;
        self.install_modules(modules, sim_response.total_required_funds.as_ref())
    }

    pub fn install_module<TInitMsg: Serialize>(
        &self,
        module_id: &str,
        init_msg: Option<&TInitMsg>,
        funds: &[Coin],
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        self.install_module_version(module_id, ModuleVersion::Latest, init_msg, funds)
    }

    pub fn install_module_version<M: Serialize>(
        &self,
        module_id: &str,
        version: ModuleVersion,
        init_msg: Option<&M>,
        funds: &[Coin],
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        self.execute(
            &ExecuteMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id(module_id, version)?,
                    init_msg.map(to_json_binary).transpose().unwrap(),
                )],
            },
            funds,
        )
        .map_err(Into::into)
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
                exec_msg: to_json_binary(&msg).unwrap(),
            },
            &[],
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
            adapter::ExecuteMsg::<Empty>::Base(adapter::BaseExecuteMsg {
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses { to_add, to_remove },
                account_adress: None,
            }),
        )?;

        Ok(())
    }

    /// Return the module info installed on the manager
    pub fn module_info(
        &self,
        module_id: &str,
    ) -> Result<Option<AccountModuleInfo>, crate::AbstractInterfaceError> {
        let module_infos = self.module_infos(None, None)?.module_infos;
        let found = module_infos
            .into_iter()
            .find(|module_info| module_info.id == module_id);
        Ok(found)
    }

    /// Get the address of a module
    /// Will err when not installed.
    pub fn module_address(
        &self,
        module_id: impl Into<String>,
    ) -> Result<Addr, crate::AbstractInterfaceError> {
        Ok(self.module_addresses(vec![module_id.into()])?.modules[0]
            .1
            .clone())
    }

    pub fn is_module_installed(
        &self,
        module_id: &str,
    ) -> Result<bool, crate::AbstractInterfaceError> {
        let module = self.module_info(module_id)?;
        Ok(module.is_some())
    }

    /// Helper to create remote accounts
    pub fn register_remote_account(
        &self,
        host_chain: TruncatedChainId,
    ) -> Result<<Chain as cw_orch::prelude::TxHandler>::Response, crate::AbstractInterfaceError>
    {
        let result = self.exec_on_module(
            to_json_binary(&abstract_std::account::ExecuteMsg::IbcAction {
                msg: abstract_std::ibc_client::ExecuteMsg::Register {
                    host_chain,
                    namespace: None,
                    install_modules: vec![ModuleInstallConfig::new(
                        ModuleInfo::from_id_latest(IBC_CLIENT)?,
                        None,
                    )],
                },
            })?,
            ACCOUNT.to_string(),
            &[],
        )?;

        Ok(result)
    }

    pub fn set_ibc_status(
        &self,
        enabled: bool,
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        let response = if enabled {
            self.install_module::<Empty>(IBC_CLIENT, None, &[])?
        } else {
            self.uninstall_module(IBC_CLIENT.to_string())?
        };

        Ok(response)
    }

    pub fn execute_on_remote(
        &self,
        host_chain: TruncatedChainId,
        msg: ExecuteMsg,
    ) -> Result<<Chain as cw_orch::prelude::TxHandler>::Response, crate::AbstractInterfaceError>
    {
        let msg = abstract_std::account::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::RemoteAction {
                host_chain,
                action: HostAction::Dispatch {
                    account_msgs: vec![msg],
                },
            },
        };

        self.execute_on_module(ACCOUNT, msg)
    }

    pub fn execute_on_remote_module(
        &self,
        host_chain: TruncatedChainId,
        module_id: &str,
        msg: Binary,
    ) -> Result<<Chain as cw_orch::prelude::TxHandler>::Response, crate::AbstractInterfaceError>
    {
        let msg = abstract_std::account::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::RemoteAction {
                host_chain,
                action: HostAction::Dispatch {
                    account_msgs: vec![ExecuteMsg::ExecOnModule {
                        module_id: module_id.to_string(),
                        exec_msg: msg,
                    }],
                },
            },
        };

        self.execute_on_module(ACCOUNT, msg)
    }

    pub fn send_all_funds_back(
        &self,
        host_chain: TruncatedChainId,
    ) -> Result<<Chain as cw_orch::prelude::TxHandler>::Response, crate::AbstractInterfaceError>
    {
        let msg = abstract_std::account::ExecuteMsg::IbcAction {
            msg: abstract_std::ibc_client::ExecuteMsg::RemoteAction {
                host_chain,
                action: HostAction::Helpers(HelperAction::SendAllBack),
            },
        };

        self.execute_on_module(ACCOUNT, msg)
    }
}
