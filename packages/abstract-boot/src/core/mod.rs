//! # Functionality we want to implement on `OS`
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

mod manager;
mod proxy;
use std::collections::HashSet;

use abstract_os::{manager::ManagerModuleInfo, objects::OsId};
use boot_core::{
    prelude::{BootUpload, ContractInstance},
    BootEnvironment,
};
use serde::Serialize;
use speculoos::prelude::*;

use crate::{get_os_core_contracts, VersionControl};

pub use self::{manager::*, proxy::*};
pub struct OS<Chain: BootEnvironment> {
    pub manager: Manager<Chain>,
    pub proxy: Proxy<Chain>,
}

impl<Chain: BootEnvironment> OS<Chain> {
    pub fn new(chain: Chain, os_id: Option<OsId>) -> Self {
        let (manager, proxy) = get_os_core_contracts(chain, os_id);
        Self { manager, proxy }
    }

    pub fn upload(&mut self) -> Result<(), crate::AbstractBootError> {
        self.manager.upload()?;
        self.proxy.upload()?;
        Ok(())
    }

    /// Register the os core contracts in the version control
    pub fn register(
        &self,
        version_control: &VersionControl<Chain>,
        version: &str,
    ) -> Result<(), crate::AbstractBootError> {
        version_control.register_core(self, version)
    }

    pub fn install_module<TInitMsg: Serialize>(
        &mut self,
        module_id: &str,
        init_msg: &TInitMsg,
    ) -> Result<(), crate::AbstractBootError> {
        self.manager.install_module(module_id, init_msg)
    }

    /// Assert that the OS has the expected modules with the provided **expected_module_addrs** installed.
    /// Also checks that the proxy's configuration includes the expected module addresses.
    /// Note that the proxy is automatically included in the assertions and *should not* (but can) be included in the expected list.
    /// Returns the `Vec<ManagerModuleInfo>` from the manager
    pub fn expect_modules(
        &self,
        module_addrs: Vec<String>,
    ) -> Result<Vec<ManagerModuleInfo>, crate::AbstractBootError> {
        let abstract_os::manager::ModuleInfosResponse {
            module_infos: manager_modules,
        } = self.manager.module_infos(None, None)?;

        let expected_module_addrs = module_addrs
            .into_iter()
            .chain(std::iter::once(self.manager.address()?.into_string()))
            .collect::<HashSet<_>>();

        // account for the proxy
        assert_that!(manager_modules).has_length(expected_module_addrs.len());

        // check proxy config
        let abstract_os::proxy::ConfigResponse {
            modules: proxy_whitelist,
        } = self.proxy.config()?;

        let actual_proxy_whitelist = HashSet::from_iter(proxy_whitelist);
        assert_eq!(actual_proxy_whitelist, expected_module_addrs);

        Ok(manager_modules)
    }
}
