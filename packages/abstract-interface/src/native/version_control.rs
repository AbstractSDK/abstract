use crate::AbstractAccount;
pub use abstract_core::version_control::{ExecuteMsgFns as VCExecFns, QueryMsgFns as VCQueryFns};
use abstract_core::{
    objects::{
        module::{Module, ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
        AccountId,
    },
    version_control::*,
    VERSION_CONTROL,
};
use cosmwasm_std::Addr;
use cw_orch::contract::Contract;
use cw_orch::interface;
#[cfg(feature = "daemon")]
use cw_orch::prelude::Daemon;
use cw_orch::prelude::*;
use semver::Version;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct VersionControl<Chain>;

impl<Chain: CwEnv> Uploadable for VersionControl<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper(&self) -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::version_control::contract::execute,
                ::version_control::contract::instantiate,
                ::version_control::contract::query,
            )
            .with_migrate(::version_control::contract::migrate),
        )
    }
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("version_control")
            .unwrap()
    }
}

impl<Chain: CwEnv> VersionControl<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn load(chain: Chain, address: &Addr) -> Self {
        Self(cw_orch::contract::Contract::new(VERSION_CONTROL, chain).with_address(Some(address)))
    }

    /// Query a single module
    pub fn module(&self, info: ModuleInfo) -> Result<Module, crate::AbstractInterfaceError> {
        let ModulesResponse { mut modules } = self.modules(vec![info])?;

        Ok(modules.swap_remove(0))
    }

    pub fn register_base(
        &self,
        account: &AbstractAccount<Chain>,
        version: &str,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let manager = account.manager.as_instance();
        let manager_module = (
            ModuleInfo::from_id(&manager.id, ModuleVersion::Version(version.to_string()))?,
            ModuleReference::AccountBase(manager.code_id()?),
        );
        self.propose_modules(vec![manager_module])?;

        log::info!("Module {} registered", manager.id);

        let proxy = account.proxy.as_instance();
        let proxy_module = (
            ModuleInfo::from_id(&proxy.id, ModuleVersion::Version(version.to_string()))?,
            ModuleReference::AccountBase(proxy.code_id()?),
        );
        self.propose_modules(vec![proxy_module])?;

        log::info!("Module {} registered", proxy.id);
        Ok(())
    }

    /// Register account modules
    pub fn register_account_mods(
        &self,
        apps: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(apps, version, |c| {
            ModuleReference::AccountBase(c.code_id().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    /// Register native modules
    pub fn register_natives(
        &self,
        natives: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(natives, version, |c| {
            ModuleReference::Native(c.address().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    pub fn register_apps(
        &self,
        apps: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(apps, version, |c| {
            ModuleReference::App(c.code_id().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    pub fn register_adapters(
        &self,
        adapters: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(adapters, version, |c| {
            ModuleReference::Adapter(c.address().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    pub fn register_standalones(
        &self,
        standalones: Vec<&Contract<Chain>>,
        version: &Version,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(standalones, version, |c| {
            ModuleReference::Standalone(c.code_id().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    fn contracts_into_module_entries<RefFn>(
        &self,
        modules: Vec<&Contract<Chain>>,
        version: &Version,
        ref_fn: RefFn,
    ) -> Result<Vec<(ModuleInfo, ModuleReference)>, crate::AbstractInterfaceError>
    where
        RefFn: Fn(&&Contract<Chain>) -> ModuleReference,
    {
        let modules_to_register: Result<
            Vec<(ModuleInfo, ModuleReference)>,
            crate::AbstractInterfaceError,
        > = modules
            .iter()
            .map(|contract| {
                Ok((
                    ModuleInfo::from_id(&contract.id, ModuleVersion::Version(version.to_string()))?,
                    ref_fn(contract),
                ))
            })
            .collect();
        modules_to_register
    }

    pub fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<AccountBase, crate::AbstractInterfaceError> {
        let resp: AccountBaseResponse = self.query(&QueryMsg::AccountBase { account_id })?;
        Ok(resp.account_base)
    }

    /// Retrieves an Adapter's address from version control given the module **id** and **version**.
    pub fn get_adapter_addr(
        &self,
        id: &str,
        version: ModuleVersion,
    ) -> Result<Addr, crate::AbstractInterfaceError> {
        let module: Module = self.module(ModuleInfo::from_id(id, version)?)?;

        Ok(module.reference.unwrap_adapter()?)
    }

    /// Retrieves an APP's code id from version control given the module **id** and **version**.
    pub fn get_app_code(
        &self,
        id: &str,
        version: ModuleVersion,
    ) -> Result<u64, crate::AbstractInterfaceError> {
        let module: Module = self.module(ModuleInfo::from_id(id, version)?)?;

        Ok(module.reference.unwrap_app()?)
    }
}

#[cfg(feature = "daemon")]
impl VersionControl<Daemon> {
    // pub fn update_code_ids(&self, new_version: Version) -> anyhow::Result<()> {
    //     let code_ids = self.get_chain().state().get_all_code_ids()?;
    //     for (contract_id, code_id) in code_ids {
    //         if NATIVE_CONTRACTS.contains(&contract_id.as_str()) {
    //             continue;
    //         }

    //         // Get latest code id
    //         let resp: Result<QueryCodeIdResponse, crate::AbstractBootError> = self.query(&QueryMsg::CodeId {
    //             module: ModuleInfo {
    //                 name: contract_id.clone(),
    //                 version: None,
    //             },
    //         });
    //         log::debug!("{:?}", resp);
    //         if new_version.pre.is_empty() {
    //             match resp {
    //                 Ok(resp) => {
    //                     let registered_code_id = resp.code_id.u64();
    //                     // If equal, continue
    //                     if registered_code_id == code_id {
    //                         continue;
    //                     } else {
    //                         let latest_version = resp.info.version;
    //                         version = latest_version.parse().unwrap();
    //                         // bump patch
    //                         version.patch += 1;
    //                     }
    //                 }
    //                 Err(_) => (),
    //             };
    //         }

    //         self.execute(
    //             &ExecuteMsg::AddCodeId {
    //                 module: contract_id.to_string(),
    //                 version: version.to_string(),
    //                 code_id,
    //             },
    //             None,
    //         )?;
    //     }
    //     Ok(())
    // }

    // pub fn update_adapters(&self) -> anyhow::Result<()> {
    //     for contract_name in chain_state.keys() {
    //         if !API_CONTRACTS.contains(&contract_name.as_str()) {
    //             continue;
    //         }

    //         // Get local addr
    //         let address: String = chain_state[contract_name].as_str().unwrap().into();

    //         // Get latest addr
    //         let resp: Result<QueryApiAddressResponse, crate::AbstractBootError> =
    //             self.query(&QueryMsg::ApiAddress {
    //                 module: ModuleInfo {
    //                     name: contract_name.clone(),
    //                     version: None,
    //                 },
    //             });
    //         log::debug!("{:?}", resp);
    //         let mut version = self.deployment_version.clone();
    //         match resp {
    //             Ok(resp) => {
    //                 let registered_addr = resp.address.to_string();

    //                 // If equal, continue
    //                 if registered_addr == address {
    //                     continue;
    //                 } else {
    //                     let latest_version = resp.info.version;
    //                     version = latest_version.parse().unwrap();
    //                     // bump patch
    //                     version.patch += 1;
    //                 }
    //             }
    //             Err(_) => (),
    //         };

    //         self.execute(
    //             &ExecuteMsg::AddApi {
    //                 module: contract_name.to_string(),
    //                 version: version.to_string(),
    //                 address,
    //             },
    //             None,
    //         )?;
    //     }
    //     Ok(())
    // }
}
