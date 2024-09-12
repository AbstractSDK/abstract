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

use abstract_std::{
    account::ModuleInstallConfig,
    objects::{
        module::{ModuleInfo, ModuleStatus, ModuleVersion},
        TruncatedChainId,
    },
    version_control::{ExecuteMsgFns, ModuleFilter, QueryMsgFns},
    ABSTRACT_EVENT_TYPE, ACCOUNT,
};
use cosmwasm_std::{from_json, to_json_binary};
use cw2::{ContractVersion, CONTRACT};
use semver::{Version, VersionReq};

use crate::{Abstract, AbstractInterfaceError, AccountDetails, AdapterDeployer};

mod account;

use std::collections::HashSet;

use abstract_std::{account::AccountModuleInfo, objects::AccountId};
use cw_orch::{environment::Environment, prelude::*};
use serde::Serialize;
use speculoos::prelude::*;

pub use self::account::*;
use crate::{get_account_contracts, VersionControl};

#[derive(Clone)]
pub struct AbstractAccount<Chain: CwEnv> {
    // TODO: merge this account with AbstractAccount
    pub account: Account<Chain>,
}

// Auto-dereference itself to have similar api with `abstract_client::Account`
impl<Chain: CwEnv> AsRef<AbstractAccount<Chain>> for AbstractAccount<Chain> {
    fn as_ref(&self) -> &AbstractAccount<Chain> {
        self
    }
}

impl<Chain: CwEnv> AbstractAccount<Chain> {
    pub fn upload(&mut self) -> Result<(), crate::AbstractInterfaceError> {
        self.account.upload()?;
        Ok(())
    }
}

impl<Chain: CwEnv> AbstractAccount<Chain> {
    pub fn new(abstract_deployment: &Abstract<Chain>, account_id: AccountId) -> Self {
        let account = get_account_contracts(&abstract_deployment.version_control, account_id);
        Self { account }
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
        funds: &[Coin],
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        self.account.install_module(module_id, init_msg, funds)
    }

    pub fn install_modules(
        &self,
        modules: Vec<ModuleInstallConfig>,
        funds: &[Coin],
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        self.account
            .install_modules(modules, funds)
            .map_err(Into::into)
    }

    pub fn install_modules_auto(
        &self,
        modules: Vec<ModuleInstallConfig>,
    ) -> Result<Chain::Response, crate::AbstractInterfaceError> {
        self.account.install_modules_auto(modules)
    }

    /// Assert that the Account has the expected modules with the provided **expected_module_addrs** installed.
    /// Note that the proxy is automatically included in the assertions.
    /// Returns the `Vec<AccountModuleInfo>` from the manager
    pub fn expect_modules(
        &self,
        module_addrs: Vec<String>,
    ) -> Result<Vec<AccountModuleInfo>, crate::AbstractInterfaceError> {
        let abstract_std::account::ModuleInfosResponse {
            module_infos: manager_modules,
        } = self.account.module_infos(None, None)?;

        // insert proxy in expected module addresses
        let expected_module_addrs = module_addrs
            .into_iter()
            .map(Addr::unchecked)
            .chain(std::iter::once(self.account.address()?))
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
        let module = self.account.module_info(module_id)?;
        Ok(module.is_some())
    }

    /// Checks that the proxy's whitelist includes the expected module addresses.
    /// Automatically includes the manager in the expected whitelist.
    pub fn expect_whitelist(
        &self,
        whitelisted_addrs: Vec<String>,
    ) -> Result<Vec<(String, Addr)>, crate::AbstractInterfaceError> {
        // insert manager in expected whitelisted addresses
        let expected_whitelisted_addrs = whitelisted_addrs
            .into_iter()
            .chain(std::iter::once(self.account.address()?.into_string()))
            .collect::<HashSet<_>>();

        // check proxy config
        let abstract_std::account::ConfigResponse {
            modules: whitelist, ..
        } = self.account.config()?;

        let actual_whitelist = HashSet::from_iter(whitelist.iter().map(|a| a.0.clone()));
        assert_eq!(actual_whitelist, expected_whitelisted_addrs);

        Ok(whitelist)
    }

    /// Gets the account ID of the account.
    pub fn id(&self) -> Result<AccountId, crate::AbstractInterfaceError> {
        Ok(self.account.config()?.account_id)
    }

    /// Installs an adapter from an adapter object
    pub fn install_adapter<CustomInitMsg: Serialize, T: AdapterDeployer<Chain, CustomInitMsg>>(
        &self,
        module: &T,
        funds: &[Coin],
    ) -> Result<Addr, crate::AbstractInterfaceError> {
        self.install_module_parse_addr::<Empty, _>(module, None, funds)
    }

    /// Installs an app from an app object
    pub fn install_app<CustomInitMsg: Serialize, T: ContractInstance<Chain>>(
        &self,
        module: &T,
        custom_init_msg: &CustomInitMsg,
        funds: &[Coin],
    ) -> Result<Addr, crate::AbstractInterfaceError> {
        // retrieve the deployment
        self.install_module_parse_addr(module, Some(&custom_init_msg), funds)
    }

    /// Installs an standalone from an standalone object
    pub fn install_standalone<CustomInitMsg: Serialize, T: ContractInstance<Chain>>(
        &self,
        standalone: &T,
        custom_init_msg: &CustomInitMsg,
        funds: &[Coin],
    ) -> Result<Addr, crate::AbstractInterfaceError> {
        // retrieve the deployment
        self.install_module_parse_addr(standalone, Some(&custom_init_msg), funds)
    }

    fn install_module_parse_addr<InitMsg: Serialize, T: ContractInstance<Chain>>(
        &self,
        module: &T,
        init_msg: Option<&InitMsg>,
        funds: &[Coin],
    ) -> Result<Addr, crate::AbstractInterfaceError> {
        let resp = self.install_module(&module.id(), init_msg, funds)?;
        let module_address = resp.event_attr_value(ABSTRACT_EVENT_TYPE, "new_modules")?;
        let module_address = Addr::unchecked(module_address);

        module.set_address(&module_address);
        Ok(module_address)
    }

    pub fn register_remote_account(
        &self,
        host_chain: TruncatedChainId,
    ) -> Result<<Chain as cw_orch::prelude::TxHandler>::Response, crate::AbstractInterfaceError>
    {
        self.account.register_remote_account(host_chain)
    }

    pub fn create_remote_account(
        &self,
        account_details: AccountDetails,
        host_chain: TruncatedChainId,
    ) -> Result<<Chain as cw_orch::prelude::TxHandler>::Response, crate::AbstractInterfaceError>
    {
        let AccountDetails {
            namespace,
            install_modules,
            // Unused fields
            name: _,
            description: _,
            link: _,
            account_id: _,
        } = account_details;

        self.account.execute_on_module(
            abstract_std::ACCOUNT,
            abstract_std::account::ExecuteMsg::IbcAction {
                msg: abstract_std::ibc_client::ExecuteMsg::Register {
                    host_chain,
                    namespace,
                    install_modules,
                },
            },
        )
    }

    pub fn create_sub_account(
        &self,
        account_details: AccountDetails,
        funds: &[Coin],
    ) -> Result<AbstractAccount<Chain>, crate::AbstractInterfaceError> {
        let AccountDetails {
            name,
            description,
            link,
            namespace,
            install_modules,
            account_id,
        } = account_details;

        let result = self.account.execute(
            &abstract_std::account::ExecuteMsg::CreateSubAccount {
                name,
                description,
                link,
                namespace,
                install_modules,
                account_id,
            },
            funds,
        )?;

        Self::from_tx_response(self.account.environment(), result)
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
        let account = Account::new_from_id(&id, chain.clone());

        // set addresses
        let account_address = result.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
        account.set_address(&Addr::unchecked(account_address));

        Ok(AbstractAccount { account })
    }

    pub fn upload_and_register_if_needed(
        &self,
        version_control: &VersionControl<Chain>,
    ) -> Result<bool, AbstractInterfaceError> {
        let mut modules_to_register = Vec::with_capacity(2);

        if self.account.upload_if_needed()?.is_some() {
            modules_to_register.push((
                self.account.as_instance(),
                ::account::contract::CONTRACT_VERSION.to_string(),
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

    /// Attempts to upgrade the Account
    /// returns `true` if any migrations were performed.
    pub fn upgrade(
        &self,
        abstract_deployment: &Abstract<Chain>,
    ) -> Result<bool, AbstractInterfaceError> {
        let mut one_migration_was_successful = false;

        // upgrade sub accounts first
        {
            let mut sub_account_ids = vec![];
            let mut start_after = None;
            loop {
                let sub_account_ids_page = self
                    .account
                    .sub_account_ids(None, start_after)?
                    .sub_accounts;

                start_after = sub_account_ids_page.last().cloned();
                if sub_account_ids_page.is_empty() {
                    break;
                }
                sub_account_ids.extend(sub_account_ids_page);
            }
            dbg!(&sub_account_ids);
            for sub_account_id in sub_account_ids {
                let abstract_account =
                    AbstractAccount::new(abstract_deployment, AccountId::local(sub_account_id));
                if abstract_account.upgrade(abstract_deployment)? {
                    one_migration_was_successful = true;
                }
            }
        }

        // We upgrade the account to the latest version through all the versions
        loop {
            if self.upgrade_next_module_version(ACCOUNT)?.is_none() {
                break;
            }
            one_migration_was_successful = true;
        }

        Ok(one_migration_was_successful)
    }

    /// Attempt to upgrade a module to its next version.
    /// Will return `Ok(None)` if the module is on its latest version already.
    fn upgrade_next_module_version(
        &self,
        module_id: &str,
    ) -> Result<Option<Chain::Response>, AbstractInterfaceError> {
        let chain = self.account.environment().clone();

        // We start by getting the current module version
        let current_cw2_module_version: ContractVersion = if module_id == ACCOUNT {
            let current_account_version = chain
                .wasm_querier()
                .raw_query(&self.account.address()?, CONTRACT.as_slice().to_vec())
                .unwrap();
            from_json(current_account_version)?
        } else {
            self.account
                .module_versions(vec![module_id.to_string()])?
                .versions[0]
                .clone()
        };
        let current_module_version = Version::parse(&current_cw2_module_version.version)?;

        let module = ModuleInfo::from_id(module_id, current_module_version.to_string().into())?;

        // We query all the module versions above the current one
        let abstr = Abstract::load_from(chain.clone())?;
        let all_next_module_versions = abstr
            .version_control
            .module_list(
                Some(ModuleFilter {
                    namespace: Some(module.namespace.to_string()),
                    name: Some(module.name.clone()),
                    version: None,
                    status: Some(ModuleStatus::Registered),
                }),
                None,
                Some(module.clone()),
            )?
            .modules
            .into_iter()
            .map(|module| {
                let version: Version = module.module.info.version.clone().try_into().unwrap();
                version
            })
            .collect::<Vec<_>>();

        // Two cases now.
        // 1. If there exists a higher non-compatible version, we want to update to the next breaking version
        // 2. If there are only compatible versions we want to update the highest compatible version

        // Set current version as version requirement (`^x.y.z`)
        let requirement = VersionReq::parse(current_module_version.to_string().as_str())?;

        // Find out the lowest next major version
        let non_compatible_versions = all_next_module_versions
            .iter()
            .filter(|version| !requirement.matches(version))
            .collect::<Vec<_>>();

        let maybe_min_non_compatible_version = non_compatible_versions.iter().min().cloned();

        let selected_version = if let Some(min_non_compatible_version) =
            maybe_min_non_compatible_version
        {
            // Case 1
            // There is a next breaking version, we want to get the highest minor version associated with it
            let requirement = VersionReq::parse(min_non_compatible_version.to_string().as_str())?;

            non_compatible_versions
                .into_iter()
                .filter(|version| requirement.matches(version))
                .max()
                .unwrap()
                .clone()
        } else {
            // Case 2
            let possible_version = all_next_module_versions
                .into_iter()
                .filter(|version| version != &current_module_version)
                .max();

            // No version upgrade required
            if possible_version.is_none() {
                return Ok(None);
            }
            possible_version.unwrap()
        };

        // Actual upgrade to the next version
        Some(self.account.upgrade(vec![(
            ModuleInfo::from_id(
                module_id,
                ModuleVersion::Version(selected_version.to_string()),
            )?,
            Some(to_json_binary(&Empty {})?),
        )]))
        .transpose()
        .map_err(Into::into)
    }

    pub fn claim_namespace(
        &self,
        namespace: impl Into<String>,
    ) -> Result<Chain::Response, AbstractInterfaceError> {
        let abstr = Abstract::load_from(self.account.environment().clone())?;
        abstr
            .version_control
            .claim_namespace(self.id()?, namespace.into())
            .map_err(Into::into)
    }
}

impl<Chain: CwEnv> std::fmt::Display for AbstractAccount<Chain> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Account: {:?} ({:?})",
            self.account.id(),
            self.account
                .addr_str()
                .or_else(|_| Result::<_, CwOrchError>::Ok(String::from("unknown"))),
        )
    }
}
