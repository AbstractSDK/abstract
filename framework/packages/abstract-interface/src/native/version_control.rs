pub use abstract_std::version_control::{ExecuteMsgFns as VCExecFns, QueryMsgFns as VCQueryFns};
use abstract_std::{
    objects::{
        module::{Module, ModuleInfo, ModuleStatus, ModuleVersion},
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

    pub fn register_base(
        &self,
        account: &AbstractAccount<Chain>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let manager = account.manager.as_instance();
        let manager_module = (
            ModuleInfo::from_id(
                &manager.id,
                ModuleVersion::Version(manager::contract::CONTRACT_VERSION.to_string()),
            )?,
            ModuleReference::AccountBase(manager.code_id()?),
        );
        self.propose_modules(vec![manager_module])?;

        log::info!("Module {} registered", manager.id);

        let proxy = account.proxy.as_instance();
        let proxy_module = (
            ModuleInfo::from_id(
                &proxy.id,
                ModuleVersion::Version(proxy::contract::CONTRACT_VERSION.to_string()),
            )?,
            ModuleReference::AccountBase(proxy.code_id()?),
        );
        self.propose_modules(vec![proxy_module])?;

        log::info!("Module {} registered", proxy.id);
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
        self.approve_any_namespace_modules(Namespace::unchecked(ABSTRACT_NAMESPACE))
    }

    /// Approve any "namespace" pending modules.
    pub fn approve_any_namespace_modules(
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

    /// Retrieves an APP or STANDALONE code id from version control given the module **id** and **version**.
    pub fn get_module_code_id(
        &self,
        id: &str,
        version: ModuleVersion,
    ) -> Result<u64, crate::AbstractInterfaceError> {
        let module: Module = self.module(ModuleInfo::from_id(id, version)?)?;

        Ok(module.reference.unwrap_code_id()?)
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
