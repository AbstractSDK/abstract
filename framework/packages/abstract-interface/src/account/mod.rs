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

use abstract_std::{manager::ModuleInstallConfig, ABSTRACT_EVENT_TYPE};

use crate::{Abstract, AbstractInterfaceError, AccountDetails, AdapterDeployer};

mod manager;
mod proxy;

use std::collections::HashSet;

use abstract_std::{manager::ManagerModuleInfo, objects::AccountId};
use cosmwasm_std::Addr;
use cw_orch::prelude::*;
use serde::Serialize;
use speculoos::prelude::*;

pub use self::{manager::*, proxy::*};
use crate::{get_account_contracts, VersionControl};

#[derive(Clone)]
pub struct AbstractAccount<Chain: CwEnv> {
    pub manager: Manager<Chain>,
    pub proxy: Proxy<Chain>,
}

// Auto-dereference itself to have similar api with `abstract_client::Account`
impl<Chain: CwEnv> AsRef<AbstractAccount<Chain>> for AbstractAccount<Chain> {
    fn as_ref(&self) -> &AbstractAccount<Chain> {
        self
    }
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
        let abstract_std::manager::ModuleInfosResponse {
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
        let abstract_std::proxy::ConfigResponse {
            modules: proxy_whitelist,
        } = self.proxy.config()?;

        let actual_proxy_whitelist = HashSet::from_iter(proxy_whitelist.clone());
        assert_eq!(actual_proxy_whitelist, expected_whitelisted_addrs);

        Ok(proxy_whitelist)
    }

    /// Gets the account ID of the account.
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
    pub fn create_sub_account(
        &self,
        account_details: AccountDetails,
        funds: Option<&[Coin]>,
    ) -> Result<AbstractAccount<Chain>, crate::AbstractInterfaceError> {
        let AccountDetails {
            name,
            description,
            link,
            namespace,
            base_asset,
            install_modules,
            account_id,
        } = account_details;

        let result = self.manager.execute(
            &abstract_std::manager::ExecuteMsg::CreateSubAccount {
                name,
                description,
                link,
                base_asset,
                namespace,
                install_modules,
                account_id,
            },
            funds,
        )?;

        Self::from_tx_response(self.manager.get_chain(), result)
    }

    // Parse account from events
    // It's restricted to parse 1 account at a time
    pub(crate) fn from_tx_response(
        chain: &Chain,
        result: <Chain as TxHandler>::Response,
    ) -> Result<AbstractAccount<Chain>, crate::AbstractInterfaceError> {
        // Parse data from events
        let acc_seq = &result.event_attr_value(ABSTRACT_EVENT_TYPE, "account_sequence")?;
        let trace = &result.event_attr_value(ABSTRACT_EVENT_TYPE, "trace")?;
        let id = AccountId::new(
            acc_seq.parse().unwrap(),
            abstract_std::objects::account::AccountTrace::try_from((*trace).as_str())?,
        )?;
        // construct manager and proxy ids
        let manager = Manager::new_from_id(&id, chain.clone());
        let proxy = Proxy::new_from_id(&id, chain.clone());

        // set addresses
        let manager_address = result.event_attr_value(ABSTRACT_EVENT_TYPE, "manager_address")?;
        manager.set_address(&Addr::unchecked(manager_address));
        let proxy_address = result.event_attr_value(ABSTRACT_EVENT_TYPE, "proxy_address")?;
        proxy.set_address(&Addr::unchecked(proxy_address));

        Ok(AbstractAccount { manager, proxy })
    }

    pub fn upload_and_register_if_needed(
        &self,
        version_control: &VersionControl<Chain>,
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

impl<Chain: CwEnv> std::fmt::Display for AbstractAccount<Chain> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Account manager: {:?} ({:?}) proxy: {:?} ({:?})",
            self.manager.id(),
            self.manager
                .addr_str()
                .or_else(|_| Result::<_, CwOrchError>::Ok(String::from("unknown"))),
            self.proxy.id(),
            self.proxy
                .addr_str()
                .or_else(|_| Result::<_, CwOrchError>::Ok(String::from("unknown"))),
        )
    }
}
