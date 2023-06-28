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
use crate::AppDeployer;
use cw_orch::deploy::Deploy;

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
    pub fn new(abs: &Abstract<Chain>, account_id: Option<AccountId>) -> Self {
        let (manager, proxy) = get_account_contracts(&abs.version_control, account_id);
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
        init_msg: &TInitMsg,
        funds: Option<&[Coin]>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        self.manager.install_module(module_id, init_msg, funds)
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

    /// Gets the account ID from the manager account
    pub fn account_id(&self) -> Result<AccountId, crate::AbstractInterfaceError> {
        let account_id: u64 = self.manager.config()?.account_id.into();
        Ok(account_id.try_into().unwrap())
    }

    /// Installs an adapter from an adapter object
    pub fn install_adapter<CustomInitMsg: Serialize, T: AdapterDeployer<Chain, CustomInitMsg>>(
        &self,
        adapter: T,
        custom_init_msg: &CustomInitMsg,
        funds: Option<&[Coin]>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        // retrieve the deployment
        let abstr = Abstract::load_from(self.manager.get_chain().to_owned())?;

        let init_msg = abstract_core::adapter::InstantiateMsg {
            module: custom_init_msg,
            base: abstract_core::adapter::BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.address()?.into(),
                version_control_address: abstr.version_control.address()?.into(),
            },
        };
        self.install_module(&adapter.id(), &init_msg, funds)
    }

    /// Installs an app from an app object
    pub fn install_app<CustomInitMsg: Serialize, T: AppDeployer<Chain>>(
        &self,
        app: T,
        custom_init_msg: &CustomInitMsg,
        funds: Option<&[Coin]>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        // retrieve the deployment
        let abstr = Abstract::load_from(self.manager.get_chain().to_owned())?;

        let init_msg = abstract_core::app::InstantiateMsg {
            module: custom_init_msg,
            base: abstract_core::app::BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.address()?.into(),
            },
        };
        self.install_module(&app.id(), &init_msg, funds)
    }
}
