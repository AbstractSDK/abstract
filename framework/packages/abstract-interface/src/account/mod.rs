//! # Functionality we want to implement on an `Account`
//!
//! ## Queries
//! - module address
//! - module asserts
//! - proxy balance
//! - proxy asserts
//! ## Actions
//! - get
//! - install module
//! - uninstall module
//! - upgrade module

use crate::Abstract;
use crate::AdapterDeployer;
use abstract_core::manager::ModuleInstallConfig;
use abstract_core::ABSTRACT_EVENT_TYPE;

mod manager;
mod proxy;

use std::collections::HashSet;

use abstract_core::{manager::ManagerModuleInfo, objects::AccountId};
use cosmwasm_std::Addr;
use cw_orch::prelude::*;
use serde::Serialize;
use speculoos::prelude::*;

use crate::{get_account_contracts, VersionControl};

pub use self::{manager::*, proxy::*};

#[derive(Clone)]
pub struct AbstractAccount<Chain: CwEnv> {
    pub manager: Manager<Chain>,
    pub proxy: Proxy<Chain>,
}

impl<Chain: CwEnv> AbstractAccount<Chain> {
    pub fn upload(&mut self) -> Result<(), crate::AbstractInterfaceError> {
        self.manager.upload()?;
        self.proxy.upload()?;
        Ok(())
    }
}

impl<Chain: CwEnv> AbstractAccount<Chain> {
    pub fn new(abstract_deployment: &Abstract<Chain>, account_id: AccountId) -> Self {
        let (manager, proxy) =
            get_account_contracts(&abstract_deployment.version_control, account_id);
        Self { manager, proxy }
    }

    /// Register the account core contracts in the version control
    pub fn register(
        &self,
        version_control: &VersionControl<Chain>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        version_control.register_base(self)
    }

    pub fn install_module<TInitMsg: Serialize>(
        &self,
        module_id: &str,
        init_msg: Option<&TInitMsg>,
        funds: Option<&[Coin]>,
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        self.manager.install_module(module_id, init_msg, funds)
    }

    pub fn install_modules(
        &self,
        modules: Vec<ModuleInstallConfig>,
        funds: Option<&[Coin]>,
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        self.manager
            .install_modules(modules, funds)
            .map_err(Into::into)
    }

    pub fn install_modules_auto(
        &self,
        modules: Vec<ModuleInstallConfig>,
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        self.manager.install_modules_auto(modules)
    }

    /// Assert that the Account has the expected modules with the provided **expected_module_addrs** installed.
    /// Note that the proxy is automatically included in the assertions.
    /// Returns the `Vec<ManagerModuleInfo>` from the manager
    pub fn expect_modules(
        &self,
        module_addrs: Vec<String>,
    ) -> Result<Vec<ManagerModuleInfo>, crate::AbstractInterfaceError> {
        let abstract_core::manager::ModuleInfosResponse {
            module_infos: manager_modules,
        } = self.manager.module_infos(None, None)?;

        // insert proxy in expected module addresses
        let expected_module_addrs = module_addrs
            .into_iter()
            .map(Addr::unchecked)
            .chain(std::iter::once(self.proxy.address()?))
            .collect::<HashSet<_>>();

        let actual_module_addrs = manager_modules
            .iter()
            .map(|module_info| module_info.address.clone())
            .collect::<HashSet<_>>();

        // assert that these modules are installed
        assert_that!(expected_module_addrs).is_equal_to(actual_module_addrs);

        Ok(manager_modules)
    }

    pub fn is_module_installed(
        &self,
        module_id: &str,
    ) -> Result<bool, crate::AbstractInterfaceError> {
        let module = self.manager.module_info(module_id)?;
        Ok(module.is_some())
    }

    /// Checks that the proxy's whitelist includes the expected module addresses.
    /// Automatically includes the manager in the expected whitelist.
    pub fn expect_whitelist(
        &self,
        whitelisted_addrs: Vec<String>,
    ) -> Result<Vec<String>, crate::AbstractInterfaceError> {
        // insert manager in expected whitelisted addresses
        let expected_whitelisted_addrs = whitelisted_addrs
            .into_iter()
            .chain(std::iter::once(self.manager.address()?.into_string()))
            .collect::<HashSet<_>>();

        // check proxy config
        let abstract_core::proxy::ConfigResponse {
            modules: proxy_whitelist,
        } = self.proxy.config()?;

        let actual_proxy_whitelist = HashSet::from_iter(proxy_whitelist.clone());
        assert_eq!(actual_proxy_whitelist, expected_whitelisted_addrs);

        Ok(proxy_whitelist)
    }

    /// Gets the account ID of the account in the local store.
    pub fn id(&self) -> Result<AccountId, crate::AbstractInterfaceError> {
        Ok(self.manager.config()?.account_id)
    }

    /// Installs an adapter from an adapter object
    pub fn install_adapter<CustomInitMsg: Serialize, T: AdapterDeployer<Chain, CustomInitMsg>>(
        &self,
        adapter: &T,
        funds: Option<&[Coin]>,
    ) -> Result<Addr, crate::AbstractInterfaceError> {
        self.install_module_parse_addr::<Empty, _>(adapter, None, funds)
    }

    /// Installs an app from an app object
    pub fn install_app<CustomInitMsg: Serialize, T: ContractInstance<Chain>>(
        &self,
        app: &T,
        custom_init_msg: &CustomInitMsg,
        funds: Option<&[Coin]>,
    ) -> Result<Addr, crate::AbstractInterfaceError> {
        // retrieve the deployment
        self.install_module_parse_addr(app, Some(&custom_init_msg), funds)
    }

    fn install_module_parse_addr<InitMsg: Serialize, T: ContractInstance<Chain>>(
        &self,
        module: &T,
        init_msg: Option<&InitMsg>,
        funds: Option<&[Coin]>,
    ) -> Result<Addr, crate::AbstractInterfaceError> {
        let resp = self.install_module(&module.id(), init_msg, funds)?;
        let module_address = resp.event_attr_value(ABSTRACT_EVENT_TYPE, "new_modules")?;
        let module_address = Addr::unchecked(module_address);

        module.set_address(&module_address);
        Ok(module_address)
    }

    pub fn register_remote_account(
        &self,
        host_chain: &str,
    ) -> Result<<Chain as cw_orch::prelude::TxHandler>::Response, crate::AbstractInterfaceError>
    {
        self.manager.register_remote_account(host_chain)
    }
}

use crate::AbstractInterfaceError;
impl<T: CwEnv> AbstractAccount<T> {
    /// Upload and register the account core contracts in the version control if they need to be updated
    pub fn upload_and_register_if_needed(
        &self,
        version_control: &VersionControl<T>,
    ) -> Result<bool, AbstractInterfaceError> {
        let mut modules_to_register = Vec::with_capacity(2);

        if self.manager.upload_if_needed()?.is_some() {
            modules_to_register.push((
                self.manager.as_instance(),
                ::manager::contract::CONTRACT_VERSION.to_string(),
            ));
        };

        if self.proxy.upload_if_needed()?.is_some() {
            modules_to_register.push((
                self.proxy.as_instance(),
                ::proxy::contract::CONTRACT_VERSION.to_string(),
            ));
        };

        let migrated = if !modules_to_register.is_empty() {
            version_control.register_account_mods(modules_to_register)?;
            true
        } else {
            false
        };

        Ok(migrated)
    }
}
