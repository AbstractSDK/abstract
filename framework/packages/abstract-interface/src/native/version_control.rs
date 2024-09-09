pub use abstract_std::version_control::{ExecuteMsgFns as VCExecFns, QueryMsgFns as VCQueryFns};
use abstract_std::{
    objects::{
        dependency::StaticDependency,
        module::{Module, ModuleId, ModuleInfo, ModuleStatus, ModuleVersion},
        module_reference::ModuleReference,
        namespace::{Namespace, ABSTRACT_NAMESPACE},
        AccountId,
    },
    version_control::*,
    VERSION_CONTROL,
};
use cw_orch::{contract::Contract, interface, prelude::*};

use crate::AbstractAccount;

type VersionString = String;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct VersionControl<Chain>;

impl<Chain: CwEnv> Uploadable for VersionControl<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::version_control::contract::execute,
                ::version_control::contract::instantiate,
                ::version_control::contract::query,
            )
            .with_migrate(::version_control::migrate::migrate),
        )
    }
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("version_control")
            .unwrap()
    }
}

impl<Chain: CwEnv> VersionControl<Chain> {
    pub fn load(chain: Chain, address: &Addr) -> Self {
        let contract = cw_orch::contract::Contract::new(VERSION_CONTROL, chain);
        contract.set_address(address);
        Self(contract)
    }

    /// Query a single module
    pub fn module(&self, info: ModuleInfo) -> Result<Module, crate::AbstractInterfaceError> {
        let ModulesResponse { mut modules } = self.modules(vec![info])?;

        Ok(modules.swap_remove(0).module)
    }

    /// Query a single module registered or pending
    pub fn registered_or_pending_module(
        &self,
        info: ModuleInfo,
    ) -> Result<Module, crate::AbstractInterfaceError> {
        let mut module_list_response = self.module_list(
            Some(ModuleFilter {
                namespace: Some(info.namespace.to_string()),
                name: Some(info.name.clone()),
                version: Some(info.version.to_string()),
                status: Some(ModuleStatus::Registered),
            }),
            None,
            None,
        )?;

        if !module_list_response.modules.is_empty() {
            // Return if it's registered module else it's pending or neither registered or pending
            Ok(module_list_response.modules.swap_remove(0).module)
        } else {
            let mut module_list_response = self.module_list(
                Some(ModuleFilter {
                    namespace: Some(info.namespace.to_string()),
                    name: Some(info.name),
                    version: Some(info.version.to_string()),
                    status: Some(ModuleStatus::Pending),
                }),
                None,
                None,
            )?;
            if !module_list_response.modules.is_empty() {
                Ok(module_list_response.modules.swap_remove(0).module)
            } else {
                Err(crate::AbstractInterfaceError::Std(
                    cosmwasm_std::StdError::generic_err("Module not found"),
                ))
            }
        }
    }

    /// Get module status or return `None` if not deployed
    pub fn module_status(
        &self,
        info: ModuleInfo,
    ) -> Result<Option<ModuleStatus>, crate::AbstractInterfaceError> {
        let is_module_status = |m: ModuleStatus| -> Result<bool, crate::AbstractInterfaceError> {
            let is_module_status = !self
                .module_list(
                    Some(ModuleFilter {
                        namespace: Some(info.namespace.to_string()),
                        name: Some(info.name.clone()),
                        version: Some(info.version.to_string()),
                        status: Some(m),
                    }),
                    None,
                    None,
                )?
                .modules
                .is_empty();
            Ok(is_module_status)
        };

        if is_module_status(ModuleStatus::Registered)? {
            Ok(Some(ModuleStatus::Registered))
        } else if is_module_status(ModuleStatus::Pending)? {
            Ok(Some(ModuleStatus::Pending))
        } else if is_module_status(ModuleStatus::Yanked)? {
            Ok(Some(ModuleStatus::Yanked))
        } else {
            // Not deployed
            Ok(None)
        }
    }

    /// Return list of registered module versions
    pub fn module_versions(
        &self,
        module_id: ModuleId,
    ) -> Result<Vec<semver::Version>, crate::AbstractInterfaceError> {
        let parts: Vec<&str> = module_id.split(':').collect();
        if parts.len() != 2 {
            return Err(abstract_std::AbstractError::FormattingError {
                object: "module_id".to_string(),
                expected: "namespace:module".to_string(),
                actual: module_id.to_string(),
            }
            .into());
        }

        let mut start_after = None;
        let mut versions: Vec<semver::Version> = vec![];
        loop {
            let modules_page = self
                .module_list(
                    Some(ModuleFilter {
                        namespace: Some(parts[0].to_owned()),
                        name: Some(parts[1].to_owned()),
                        version: None,
                        status: Some(ModuleStatus::Registered),
                    }),
                    None,
                    start_after,
                )?
                .modules;
            if modules_page.is_empty() {
                break;
            }
            start_after = modules_page.last().map(|module| module.module.info.clone());
            let versions_page = modules_page
                .into_iter()
                .map(|module| module.module.info.version.try_into().unwrap());

            versions.extend(versions_page)
        }
        Ok(versions)
    }

    // Check that module dependencies deployed on chain
    pub fn assert_dependencies_deployed(
        &self,
        dependencies: &[StaticDependency],
    ) -> Result<(), crate::AbstractInterfaceError> {
        for dependency in dependencies {
            let module_versions = self.module_versions(dependency.id)?;
            // Check if at least one version matches
            let matches = module_versions
                .iter()
                .any(|version| dependency.matches(version));
            if !matches {
                return Err(crate::AbstractInterfaceError::NoMatchingModule(
                    dependency.clone(),
                ));
            }
        }
        Ok(())
    }

    pub fn register_base(
        &self,
        account: &AbstractAccount<Chain>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let account = account.account.as_instance();
        let account_module = (
            ModuleInfo::from_id(
                &account.id,
                ModuleVersion::Version(manager::contract::CONTRACT_VERSION.to_string()),
            )?,
            ModuleReference::AccountBase(account.code_id()?),
        );
        self.propose_modules(vec![account_module])?;

        log::info!("Module {} registered", account.id);
        Ok(())
    }

    /// Register account modules
    pub fn register_account_mods(
        &self,
        apps: Vec<(&Contract<Chain>, VersionString)>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(apps, |c| {
            ModuleReference::AccountBase(c.code_id().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    /// Register native modules
    pub fn register_natives(
        &self,
        natives: Vec<(&Contract<Chain>, VersionString)>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(natives, |c| {
            ModuleReference::Native(c.address().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    /// Register services modules
    pub fn register_services(
        &self,
        services: Vec<(&Contract<Chain>, VersionString)>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(services, |c| {
            ModuleReference::Service(c.address().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    pub fn register_apps(
        &self,
        apps: Vec<(&Contract<Chain>, VersionString)>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self
            .contracts_into_module_entries(apps, |c| ModuleReference::App(c.code_id().unwrap()))?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    pub fn register_adapters(
        &self,
        adapters: Vec<(&Contract<Chain>, VersionString)>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(adapters, |c| {
            ModuleReference::Adapter(c.address().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    pub fn register_standalones(
        &self,
        standalones: Vec<(&Contract<Chain>, VersionString)>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let to_register = self.contracts_into_module_entries(standalones, |c| {
            ModuleReference::Standalone(c.code_id().unwrap())
        })?;
        self.propose_modules(to_register)?;
        Ok(())
    }

    /// Approve any abstract-namespaced pending modules.
    pub fn approve_any_abstract_modules(&self) -> Result<(), crate::AbstractInterfaceError> {
        self.approve_all_modules_for_namespace(Namespace::unchecked(ABSTRACT_NAMESPACE))
    }

    /// Approve any "namespace" pending modules.
    pub fn approve_all_modules_for_namespace(
        &self,
        namespace: Namespace,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let proposed_namespace_modules = self.module_list(
            Some(ModuleFilter {
                namespace: Some(namespace.to_string()),
                status: Some(ModuleStatus::Pending),
                ..Default::default()
            }),
            None,
            None,
        )?;

        if proposed_namespace_modules.modules.is_empty() {
            return Ok(());
        }

        self.approve_or_reject_modules(
            proposed_namespace_modules
                .modules
                .into_iter()
                .map(|m| m.module.info)
                .collect(),
            vec![],
        )?;
        Ok(())
    }

    fn contracts_into_module_entries<RefFn>(
        &self,
        modules: Vec<(&Contract<Chain>, VersionString)>,
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
            .map(|(contract, version)| {
                Ok((
                    ModuleInfo::from_id(&contract.id, ModuleVersion::Version(version.to_owned()))?,
                    ref_fn(contract),
                ))
            })
            .collect();
        modules_to_register
    }

    pub fn get_account(
        &self,
        account_id: AccountId,
    ) -> Result<Account, crate::AbstractInterfaceError> {
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

    /// Retrieves an APP's code id from version control given the module **id** and **version**.
    pub fn get_standalone_code(
        &self,
        id: &str,
        version: ModuleVersion,
    ) -> Result<u64, crate::AbstractInterfaceError> {
        let module: Module = self.module(ModuleInfo::from_id(id, version)?)?;

        Ok(module.reference.unwrap_standalone()?)
    }
}

impl VersionControl<Mock> {
    /// Approve any pending modules.
    pub fn approve_any_modules(&self) -> Result<(), crate::AbstractInterfaceError> {
        let proposed_abstract_modules = self.module_list(
            Some(ModuleFilter {
                status: Some(ModuleStatus::Pending),
                ..Default::default()
            }),
            None,
            None,
        )?;

        if proposed_abstract_modules.modules.is_empty() {
            return Ok(());
        }

        let owner = self.ownership()?;
        self.call_as(&Addr::unchecked(owner.owner.unwrap()))
            .approve_or_reject_modules(
                proposed_abstract_modules
                    .modules
                    .iter()
                    .map(|m| m.module.info.clone())
                    .collect(),
                vec![],
            )?;
        Ok(())
    }
}
