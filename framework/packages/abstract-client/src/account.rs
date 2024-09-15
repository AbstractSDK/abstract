//! # Represents Abstract Account
//!
//! [`Account`] allows you to interact with your or another user Abstract Account
//!
//! Example of creating an account
//! ```
//! # use abstract_client::AbstractClientError;
//! use abstract_client::{AbstractClient, Account};
//! use cw_orch::prelude::*;
//!
//! # let chain = MockBech32::new("mock");
//! # let client: AbstractClient<MockBech32> = AbstractClient::builder(chain).build()?;
//!
//! let alice_account: Account<MockBech32> = client
//!     .account_builder()
//!     .name("Alice")
//!     .build()?;
//!
//! assert_eq!(alice_account.owner()?, client.sender());
//! # Ok::<(), AbstractClientError>(())
//! ```
use std::fmt::{Debug, Display};

use abstract_interface::{
    Abstract, AbstractInterfaceError, AccountDetails, AccountI, DependencyCreation, IbcClient,
    InstallConfig, MFactoryQueryFns, ManagerExecFns, ManagerQueryFns, RegisteredModule, VCQueryFns,
};
use abstract_std::{
    account,
    account::{
        state::AccountInfo, AccountModuleInfo, InfoResponse, ModuleAddressesResponse,
        ModuleInfosResponse, ModuleInstallConfig,
    },
    objects::{
        gov_type::GovernanceDetails,
        module::{ModuleId, ModuleInfo, ModuleVersion},
        namespace::Namespace,
        ownership,
        validation::verifiers,
        AccountId,
    },
    version_control::{self, NamespaceResponse},
    ACCOUNT, IBC_CLIENT,
};
use cosmwasm_std::{to_json_binary, Attribute, Coins, CosmosMsg, Uint128};
use cw_orch::{
    contract::Contract,
    environment::{Environment as _, MutCwEnv},
    prelude::*,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    client::AbstractClientResult, infrastructure::Infrastructure, AbstractClientError, Application,
    Environment,
};

/// A builder for creating [`Accounts`](Account).
/// Get the builder from the [`AbstractClient::account_builder`](crate::AbstractClient)
/// and create the account with the `build` method.
///
/// ```
/// # use cw_orch::prelude::*;
/// # use abstract_client::{AbstractClientError, Environment};
/// # let chain = MockBech32::new("mock");
/// # let abstr_client = abstract_client::AbstractClient::builder(chain).build().unwrap();
/// # let chain = abstr_client.environment();
/// use abstract_client::{AbstractClient, Account};
///
/// let client = AbstractClient::new(chain)?;
/// let account: Account<MockBech32> = client.account_builder()
///     .name("alice")
///     // other account configuration
///     .build()?;
/// # Ok::<(), AbstractClientError>(())
/// ```
pub struct AccountBuilder<'a, Chain: CwEnv> {
    pub(crate) abstr: &'a Abstract<Chain>,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    namespace: Option<Namespace>,
    // TODO: Decide if we want to abstract this as well.
    ownership: Option<GovernanceDetails<String>>,
    owner_account: Option<&'a Account<Chain>>,
    install_modules: Vec<ModuleInstallConfig>,
    enable_ibc: bool,
    funds: AccountCreationFunds,
    fetch_if_namespace_claimed: bool,
    install_on_sub_account: bool,
    expected_local_account_id: Option<u32>,
}

/// Creation funds
enum AccountCreationFunds {
    #[allow(clippy::type_complexity)]
    Auto(Box<dyn Fn(&[Coin]) -> bool>),
    Coins(Coins),
}

impl<'a, Chain: CwEnv> AccountBuilder<'a, Chain> {
    pub(crate) fn new(abstr: &'a Abstract<Chain>) -> Self {
        Self {
            abstr,
            name: None,
            description: None,
            link: None,
            namespace: None,
            ownership: None,
            owner_account: None,
            install_modules: vec![],
            enable_ibc: false,
            funds: AccountCreationFunds::Coins(Coins::default()),
            fetch_if_namespace_claimed: true,
            install_on_sub_account: false,
            expected_local_account_id: None,
        }
    }

    /// Username for the account
    /// Defaults to "Default Abstract Account"
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    /// Description for the account
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    /// http(s) or ipfs link for the account
    pub fn link(&mut self, link: impl Into<String>) -> &mut Self {
        self.link = Some(link.into());
        self
    }

    /// Unique namespace for the account
    /// Setting this will claim the namespace for the account on construction.
    pub fn namespace(&mut self, namespace: Namespace) -> &mut Self {
        self.namespace = Some(namespace);
        self
    }

    /// Try to fetch the account before creating it if the namespace is already claimed.
    /// Defaults to `true`.
    pub fn fetch_if_namespace_claimed(&mut self, value: bool) -> &mut Self {
        self.fetch_if_namespace_claimed = value;
        self
    }

    /// Install modules on a new sub-account instead of current account.
    /// Defaults to `true`
    pub fn install_on_sub_account(&mut self, value: bool) -> &mut Self {
        self.install_on_sub_account = value;
        self
    }

    /// Create sub-account instead
    /// And set install_on_sub_account to false to prevent installing on sub account of the sub account
    pub fn sub_account(&mut self, owner_account: &'a Account<Chain>) -> &mut Self {
        self.owner_account = Some(owner_account);
        self
    }

    /// Governance of the account.
    /// Defaults to the [`GovernanceDetails::Monarchy`] variant, owned by the sender
    pub fn ownership(&mut self, ownership: GovernanceDetails<String>) -> &mut Self {
        self.ownership = Some(ownership);
        self
    }

    /// Install an adapter on current account.
    pub fn install_adapter<M: InstallConfig<InitMsg = Empty>>(
        &mut self,
    ) -> AbstractClientResult<&mut Self> {
        self.install_modules.push(M::install_config(&Empty {})?);
        Ok(self)
    }

    /// Install an application on current account.
    pub fn install_app<M: InstallConfig>(
        &mut self,
        configuration: &M::InitMsg,
    ) -> AbstractClientResult<&mut Self> {
        self.install_modules.push(M::install_config(configuration)?);
        Ok(self)
    }

    /// Install an standalone on current account.
    pub fn install_standalone<M: InstallConfig>(
        &mut self,
        configuration: &M::InitMsg,
    ) -> AbstractClientResult<&mut Self> {
        self.install_modules.push(M::install_config(configuration)?);
        Ok(self)
    }

    /// Install an service on current account.
    pub fn install_service<M: InstallConfig>(
        &mut self,
        configuration: &M::InitMsg,
    ) -> AbstractClientResult<&mut Self> {
        self.install_modules.push(M::install_config(configuration)?);
        Ok(self)
    }

    /// Install an application with dependencies on current account.
    pub fn install_app_with_dependencies<M: DependencyCreation + InstallConfig>(
        &mut self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
    ) -> AbstractClientResult<&mut Self> {
        let deps_install_config = M::dependency_install_configs(dependencies_config)?;
        self.install_modules.extend(deps_install_config);
        self.install_modules
            .push(M::install_config(module_configuration)?);
        Ok(self)
    }

    /// Install an standalone with dependencies on current account.
    pub fn install_standalone_with_dependencies<M: DependencyCreation + InstallConfig>(
        &mut self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
    ) -> AbstractClientResult<&mut Self> {
        let deps_install_config = M::dependency_install_configs(dependencies_config)?;
        self.install_modules.extend(deps_install_config);
        self.install_modules
            .push(M::install_config(module_configuration)?);
        Ok(self)
    }

    /// Install unchecked modules on current account.
    ///
    /// Since this method allows to pass unchecked `ModuleInstallConfig` \
    /// calling this method should be avoided unless it's the only way to install module, for example no `InstallModule` implemented for the module
    pub fn with_modules(
        &mut self,
        install_modules: impl IntoIterator<Item = ModuleInstallConfig>,
    ) -> &mut Self {
        self.install_modules.extend(install_modules);
        self
    }

    /// Enable ibc on account. This parameter ignored if installation of `IbcClient`
    /// already specified in `install_modules`.
    pub fn enable_ibc(&mut self) -> &mut Self {
        self.enable_ibc = true;
        self
    }

    /// Enables automatically paying for module instantiations and namespace registration.
    /// The provided function will be called with the required funds. If the function returns `false`,
    /// the account creation will fail.
    pub fn auto_fund_assert<F: Fn(&[Coin]) -> bool + 'static>(&mut self, f: F) -> &mut Self {
        self.funds = AccountCreationFunds::Auto(Box::new(f));
        self
    }

    /// Enables automatically paying for module instantiations and namespace registration.
    /// Use `auto_fund_assert` to add limits to the auto fund mode.
    pub fn auto_fund(&mut self) -> &mut Self {
        self.funds = AccountCreationFunds::Auto(Box::new(|_| true));
        self
    }

    /// Add funds to the account creation
    /// Can't be used in pair with auto fund mode
    pub fn funds(&mut self, funds: &[Coin]) -> AbstractClientResult<&mut Self> {
        let coins = match &mut self.funds {
            AccountCreationFunds::Auto(_) => return Err(AbstractClientError::FundsWithAutoFund {}),
            AccountCreationFunds::Coins(coins) => coins,
        };

        for coin in funds {
            coins
                .add(coin.clone())
                .map_err(AbstractInterfaceError::from)?;
        }
        Ok(self)
    }

    /// Assign expected local account_id on creation.
    /// It's designed to be used in pair with [`crate::AbstractClient::random_account_id`].
    /// The tx will error if this account id already claimed or it's less than 2147483648. Useful for instantiate2 address prediction.
    pub fn expected_account_id(&mut self, local_account_id: u32) -> &mut Self {
        self.expected_local_account_id = Some(local_account_id);
        self
    }

    /// Builds the [`Account`].
    pub fn build(&self) -> AbstractClientResult<Account<Chain>> {
        let install_modules = self.install_modules();
        if self.fetch_if_namespace_claimed {
            // Check if namespace already claimed
            if let Some(ref namespace) = self.namespace {
                let account_from_namespace_result: Option<Account<Chain>> =
                    Account::maybe_from_namespace(
                        self.abstr,
                        namespace.clone(),
                        self.install_on_sub_account,
                    )?;

                // Only return if the account can be retrieved without errors.
                if let Some(account_from_namespace) = account_from_namespace_result {
                    let modules_to_install =
                        account_from_namespace.missing_modules(&install_modules)?;
                    if !modules_to_install.is_empty() {
                        let funds = self.generate_funds(&modules_to_install, false)?;
                        account_from_namespace
                            .abstr_account
                            .install_modules(modules_to_install, &funds)?;
                    }
                    return Ok(account_from_namespace);
                }
            }
        }

        let chain = self.abstr.version_control.environment();
        let sender = chain.sender_addr().to_string();
        let name = self
            .name
            .clone()
            .unwrap_or_else(|| String::from("Default Abstract Account"));
        let ownership = self
            .ownership
            .clone()
            .unwrap_or(GovernanceDetails::Monarchy { monarch: sender });

        // Validate everything before sending tx
        verifiers::validate_name(&name)?;
        verifiers::validate_description(self.description.as_deref())?;
        verifiers::validate_link(self.link.as_deref())?;

        let funds = self.generate_funds(&install_modules, true)?;
        let account_details = AccountDetails {
            name,
            description: self.description.clone(),
            link: self.link.clone(),
            namespace: self.namespace.as_ref().map(ToString::to_string),
            install_modules,
        };
        let abstract_account = match self.owner_account {
            None => {
                // https://github.com/AbstractSDK/abstract/pull/446#discussion_r1756768435
                todo!()
            }
            Some(owner_account) => owner_account
                .abstr_account
                .create_sub_account_helper(account_details, &funds)?,
        };
        Ok(Account::new(abstract_account, self.install_on_sub_account))
    }

    /// Modules to install without duplicates
    fn install_modules(&self) -> Vec<ModuleInstallConfig> {
        let mut install_modules = self.install_modules.clone();
        if self.enable_ibc {
            install_modules.push(IbcClient::<Chain>::install_config(&Empty {}).unwrap());
        }
        install_modules.dedup();
        install_modules
    }

    fn generate_funds(
        &self,
        // Not using self.install_modules because it's maybe build on existent account and we pass only new modules here
        install_modules: &[ModuleInstallConfig],
        // Include funds for namespace claiming
        include_namespace_claiming: bool,
    ) -> AbstractClientResult<Vec<Coin>> {
        let funds = match &self.funds {
            AccountCreationFunds::Auto(auto_funds_assert) => {
                let modules = install_modules.iter().map(|m| m.module.clone()).collect();
                // Simulate module install to find out required funds
                let simulate_response = self
                    .abstr
                    .module_factory
                    .simulate_install_modules(modules)?;

                let mut funds = Coins::try_from(simulate_response.total_required_funds).unwrap();

                // Add namespace fee if any
                if include_namespace_claiming && self.namespace.is_some() {
                    let vc_config = self.abstr.version_control.config()?;

                    if let Some(namespace_fee) = vc_config.namespace_registration_fee {
                        funds
                            .add(namespace_fee)
                            .map_err(AbstractInterfaceError::from)?;
                    }
                };

                let funds = funds.into_vec();
                // Use auto funds assert function for validation
                if !auto_funds_assert(&funds) {
                    return Err(AbstractClientError::AutoFundsAssertFailed(funds));
                }
                funds
            }
            AccountCreationFunds::Coins(coins) => coins.to_vec(),
        };
        Ok(funds)
    }
}

/// Represents an existing Abstract account.
///
/// Get this struct from [`AbstractClient::account_from_namespace`](crate::AbstractClient)
/// or create a new account with the [`AccountBuilder`](crate::AbstractClient::account_builder).
#[derive(Clone)]
pub struct Account<Chain: CwEnv> {
    pub(crate) abstr_account: AccountI<Chain>,
    install_on_sub_account: bool,
}

impl<Chain: CwEnv> AsRef<AccountI<Chain>> for Account<Chain> {
    fn as_ref(&self) -> &AccountI<Chain> {
        &self.abstr_account
    }
}

struct ParsedAccountCreationResponse {
    sub_account_id: u32,
    module_address: String,
}

impl<Chain: CwEnv> Account<Chain> {
    pub(crate) fn new(abstract_account: AccountI<Chain>, install_on_sub_account: bool) -> Self {
        Self {
            abstr_account: abstract_account,
            install_on_sub_account,
        }
    }

    pub(crate) fn maybe_from_namespace(
        abstr: &Abstract<Chain>,
        namespace: Namespace,
        install_on_sub_account: bool,
    ) -> AbstractClientResult<Option<Self>> {
        let namespace_response: NamespaceResponse = abstr.version_control.namespace(namespace)?;

        let NamespaceResponse::Claimed(info) = namespace_response else {
            return Ok(None);
        };

        let abstract_account: AccountI<Chain> = AccountI::load_from(abstr, info.account_id);

        Ok(Some(Self::new(abstract_account, install_on_sub_account)))
    }

    /// Get the [`AccountId`] of the Account
    pub fn id(&self) -> AbstractClientResult<AccountId> {
        self.abstr_account.id().map_err(Into::into)
    }

    /// Wether this account installs his applications on a sub account
    pub fn install_on_sub_account(&self) -> bool {
        self.install_on_sub_account
    }

    /// Query account balance of a given denom
    pub fn query_balance(&self, denom: impl Into<String>) -> AbstractClientResult<Uint128> {
        let coins = self
            .environment()
            .bank_querier()
            .balance(&self.address()?, Some(denom.into()))
            .map_err(Into::into)?;

        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    /// Query account balances of all denoms
    pub fn query_balances(&self) -> AbstractClientResult<Vec<Coin>> {
        self.environment()
            .bank_querier()
            .balance(&self.address()?, None)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Query account info
    pub fn info(&self) -> AbstractClientResult<AccountInfo> {
        let info_response: InfoResponse = self.abstr_account.info()?;
        Ok(info_response.info)
    }

    /// Install an application on the account.
    /// if `install_on_sub_account` is `true`, the application will be installed on new a sub-account. (default)
    /// Errors if this module already installed
    pub fn install_app<M: InstallConfig + From<Contract<Chain>>>(
        &self,
        configuration: &M::InitMsg,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let modules = vec![M::install_config(configuration)?];

        match self.install_on_sub_account {
            true => self.install_module_sub_internal(modules, funds),
            false => self.install_module_current_internal(modules, funds),
        }
    }

    /// Install an standalone on the account.
    /// if `install_on_sub_account` is `true`, the application will be installed on new a sub-account. (default)
    /// Errors if this module already installed
    pub fn install_standalone<M: InstallConfig + From<Contract<Chain>>>(
        &self,
        configuration: &M::InitMsg,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let modules = vec![M::install_config(configuration)?];

        match self.install_on_sub_account {
            true => self.install_module_sub_internal(modules, funds),
            false => self.install_module_current_internal(modules, funds),
        }
    }

    /// Install an service on the account.
    /// if `install_on_sub_account` is `true`, the application will be installed on new a sub-account.
    /// Errors if this module already installed
    pub fn install_service<M: InstallConfig + From<Contract<Chain>>>(
        &self,
        configuration: &M::InitMsg,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let modules = vec![M::install_config(configuration)?];

        match self.install_on_sub_account {
            true => self.install_module_sub_internal(modules, funds),
            false => self.install_module_current_internal(modules, funds),
        }
    }

    /// Install an adapter on current account.
    /// Errors if this module already installed
    pub fn install_adapter<M: InstallConfig<InitMsg = Empty> + From<Contract<Chain>>>(
        &self,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let modules = vec![M::install_config(&Empty {})?];

        match self.install_on_sub_account {
            true => self.install_module_sub_internal(modules, funds),
            false => self.install_module_current_internal(modules, funds),
        }
    }

    /// Creates a new sub-account on the current account and
    /// installs an App module and its dependencies with the provided dependencies config. \
    ///
    /// The returned [`Application`] is a wrapper around the sub-account and simplifies interaction with the App module.
    /// Errors if this module installed
    /// FIXME: Errors if any of the dependencies modules already installed
    pub fn install_app_with_dependencies<
        M: DependencyCreation + InstallConfig + From<Contract<Chain>>,
    >(
        &self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let mut install_configs: Vec<ModuleInstallConfig> =
            M::dependency_install_configs(dependencies_config)?;
        install_configs.push(M::install_config(module_configuration)?);

        match self.install_on_sub_account {
            true => self.install_module_sub_internal(install_configs, funds),
            false => self.install_module_current_internal(install_configs, funds),
        }
    }

    /// Creates a new sub-account on the current account and
    /// installs an Standalone module and its dependencies with the provided dependencies config. \
    ///
    /// The returned [`Application`] is a wrapper around the sub-account and simplifies interaction with the Standalone module.
    pub fn install_standalone_with_dependencies<
        M: DependencyCreation + InstallConfig + From<Contract<Chain>>,
    >(
        &self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let mut install_configs: Vec<ModuleInstallConfig> =
            M::dependency_install_configs(dependencies_config)?;
        install_configs.push(M::install_config(module_configuration)?);

        match self.install_on_sub_account {
            true => self.install_module_sub_internal(install_configs, funds),
            false => self.install_module_current_internal(install_configs, funds),
        }
    }

    /// Upgrades the account to the latest version
    ///
    /// Migrates manager and proxy contracts to their respective new versions.
    pub fn upgrade(&self, version: ModuleVersion) -> AbstractClientResult<Chain::Response> {
        self.abstr_account
            .upgrade(vec![(
                ModuleInfo::from_id(abstract_std::registry::ACCOUNT, version.clone())?,
                Some(
                    to_json_binary(&abstract_std::account::MigrateMsg {})
                        .map_err(Into::<CwOrchError>::into)?,
                ),
            )])
            .map_err(Into::into)
    }

    /// Returns owner of the account
    pub fn ownership(&self) -> AbstractClientResult<ownership::Ownership<String>> {
        self.abstr_account.ownership().map_err(Into::into)
    }

    /// Returns the owner address of the account.
    /// If the account is a sub-account, it will return the top-level owner address.
    pub fn owner(&self) -> AbstractClientResult<Addr> {
        self.abstr_account
            .top_level_owner()
            .map(|tlo| tlo.address)
            .map_err(Into::into)
    }

    /// Executes a [`CosmosMsg`] on the proxy of the account.
    pub fn execute(
        &self,
        execute_msgs: impl IntoIterator<Item = impl Into<CosmosMsg>>,
        funds: &[Coin],
    ) -> AbstractClientResult<Chain::Response> {
        let msgs = execute_msgs.into_iter().map(Into::into).collect();
        self.configure(
            &account::ExecuteMsg::ExecOnModule {
                module_id: ACCOUNT.to_owned(),
                exec_msg: to_json_binary(&abstract_std::account::ExecuteMsg::ModuleAction { msgs })
                    .map_err(AbstractInterfaceError::from)?,
            },
            funds,
        )
    }

    /// Executes a [`account::ExecuteMsg`] on the account.
    pub fn configure(
        &self,
        execute_msg: &account::ExecuteMsg,
        funds: &[Coin],
    ) -> AbstractClientResult<Chain::Response> {
        self.abstr_account
            .execute(execute_msg, funds)
            .map_err(Into::into)
    }

    /// Queries a module on the account.
    pub fn query_module<Q: Serialize + Debug, T: Serialize + DeserializeOwned>(
        &self,
        module_id: ModuleId,
        msg: &Q,
    ) -> AbstractClientResult<T> {
        let mut module_address_response = self.module_addresses(vec![module_id.to_owned()])?;
        let (_, module_addr) = module_address_response.modules.pop().unwrap();
        let response = self
            .environment()
            .query(msg, &module_addr)
            .map_err(Into::into)?;
        Ok(response)
    }

    /// Set IBC status on an Account.
    pub fn set_ibc_status(&self, enabled: bool) -> AbstractClientResult<Chain::Response> {
        self.abstr_account
            .set_ibc_status(enabled)
            .map_err(Into::into)
    }

    /// Module infos of installed modules on account
    pub fn module_infos(&self) -> AbstractClientResult<ModuleInfosResponse> {
        let mut module_infos: Vec<AccountModuleInfo> = vec![];
        loop {
            let last_module_id: Option<String> = module_infos
                .last()
                .map(|module_info| module_info.id.clone());
            let res: ModuleInfosResponse = self.abstr_account.module_infos(None, last_module_id)?;
            if res.module_infos.is_empty() {
                break;
            }
            module_infos.extend(res.module_infos);
        }
        Ok(ModuleInfosResponse { module_infos })
    }

    /// Addresses of installed modules on account
    pub fn module_addresses(
        &self,
        ids: Vec<String>,
    ) -> AbstractClientResult<ModuleAddressesResponse> {
        self.abstr_account.module_addresses(ids).map_err(Into::into)
    }

    /// Check if module installed on account
    pub fn module_installed(&self, id: ModuleId) -> AbstractClientResult<bool> {
        // Currently this is the only way that
        // - Doesn't error on missing account module
        // - Predictable gas usage
        let key = account::state::ACCOUNT_MODULES.key(id).to_vec();
        let maybe_module_addr = self
            .environment()
            .wasm_querier()
            .raw_query(&self.abstr_account.address()?, key)
            .map_err(Into::into)?;
        Ok(!maybe_module_addr.is_empty())
    }

    /// Check if module version installed on account
    pub fn module_version_installed(&self, module: ModuleInfo) -> AbstractClientResult<bool> {
        let module_id = module.id();
        // First we need to verify it's installed or next query will fail with strange error
        if !self.module_installed(&module_id)? {
            return Ok(false);
        }

        let mut module_versions_response = self.abstr_account.module_versions(vec![module_id])?;
        let installed_version = module_versions_response.versions.pop().unwrap().version;
        let expected_version = match &module.version {
            // If latest we need to find latest version stored in VC
            ModuleVersion::Latest => {
                let manager_config = self.abstr_account.config()?;
                let mut modules_response: version_control::ModulesResponse = self
                    .environment()
                    .query(
                        &version_control::QueryMsg::Modules {
                            infos: vec![module.clone()],
                        },
                        &manager_config.version_control_address,
                    )
                    .map_err(Into::into)?;
                modules_response
                    .modules
                    .pop()
                    .unwrap()
                    .module
                    .info
                    .version
                    .to_string()
            }
            ModuleVersion::Version(version) => version.clone(),
        };
        Ok(installed_version == expected_version)
    }

    /// Check if module installed on account
    pub fn ibc_status(&self) -> AbstractClientResult<bool> {
        self.module_installed(IBC_CLIENT)
    }

    /// Get Sub Accounts of this account
    pub fn sub_accounts(&self) -> AbstractClientResult<Vec<Account<Chain>>> {
        let mut sub_accounts = vec![];
        let mut start_after = None;
        let abstr_deployment = Abstract::load_from(self.environment())?;
        loop {
            let sub_account_ids = self
                .abstr_account
                .sub_account_ids(None, start_after)?
                .sub_accounts;
            start_after = sub_account_ids.last().cloned();

            if sub_account_ids.is_empty() {
                break;
            }
            sub_accounts.extend(sub_account_ids);
        }

        Ok(sub_accounts
            .into_iter()
            .map(|id| {
                Account::new(
                    AccountI::load_from(&abstr_deployment, AccountId::local(id)),
                    false,
                )
            })
            .collect())
    }

    /// Address of the account (proxy)
    pub fn address(&self) -> AbstractClientResult<Addr> {
        Ok(self.abstr_account.address()?)
    }

    /// Retrieve installed application on account
    /// This can't retrieve sub-account installed applications.
    pub fn application<M: RegisteredModule + From<Contract<Chain>>>(
        &self,
    ) -> AbstractClientResult<Application<Chain, M>> {
        let module = self.module()?;
        let account = self.clone();

        Application::new(account, module)
    }

    /// Install module on current account
    fn install_module_current_internal<M: RegisteredModule + From<Contract<Chain>>>(
        &self,
        mut modules: Vec<ModuleInstallConfig>,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let module_infos = self.module_infos()?;
        modules.retain(|m| {
            !module_infos
                .module_infos
                .iter()
                .any(|module_info| module_info.id == m.module.id())
        });
        if !modules.is_empty() {
            self.abstr_account.install_modules(modules, funds)?;
        }

        let module = self.module::<M>()?;

        Application::new(
            Account::new(self.abstr_account.clone(), self.install_on_sub_account),
            module,
        )
    }

    /// Installs module on sub account
    fn install_module_sub_internal<M: RegisteredModule + From<Contract<Chain>>>(
        &self,
        modules: Vec<ModuleInstallConfig>,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        // Create sub account.
        let sub_account_response = self.abstr_account.create_sub_account(
            modules,
            "Sub Account".to_owned(),
            None,
            None,
            None,
            funds,
        )?;

        let parsed_account_creation_response =
            Self::parse_account_creation_response(sub_account_response);

        let sub_account: AccountI<Chain> = AccountI::load_from(
            &self.infrastructure()?,
            AccountId::local(parsed_account_creation_response.sub_account_id),
        );

        let contract = Contract::new(
            M::installed_module_contract_id(&sub_account.id()?),
            self.environment(),
        );
        contract.set_address(&Addr::unchecked(
            parsed_account_creation_response.module_address,
        ));

        let app: M = contract.into();

        Application::new(Account::new(sub_account, false), app)
    }

    fn parse_account_creation_response(response: Chain::Response) -> ParsedAccountCreationResponse {
        let wasm_abstract_attributes: Vec<Attribute> = response
            .events()
            .into_iter()
            .filter(|e| e.ty == "wasm-abstract")
            .flat_map(|e| e.attributes)
            .collect();

        let sub_account_id: Option<u32> = wasm_abstract_attributes
            .iter()
            .find(|a| a.key == "sub_account_added")
            .map(|a| a.value.parse().unwrap());

        let module_addresses: Option<String> = wasm_abstract_attributes
            .iter()
            .find(|a| a.key == "new_modules")
            .map(|a| a.value.parse().unwrap());

        // When there are multiple modules registered the addresses are returned in a common
        // separated list. We want the last one as that is the "top-level" module while the rest
        // are dependencies, since in the sub-account creation call, we pass in the top-level
        // module last.
        let module_address: String = module_addresses
            .unwrap()
            .split(',')
            .last()
            .unwrap()
            .to_string();

        ParsedAccountCreationResponse {
            // We expect both of these fields to be present.
            sub_account_id: sub_account_id.unwrap(),
            module_address,
        }
    }

    pub(crate) fn module<T: RegisteredModule + From<Contract<Chain>>>(
        &self,
    ) -> AbstractClientResult<T> {
        let module_id = T::module_id();
        let account_module_id = T::installed_module_contract_id(&self.id()?);
        let maybe_module_addr = self.module_addresses(vec![module_id.to_string()])?.modules;

        if !maybe_module_addr.is_empty() {
            let contract = Contract::new(account_module_id, self.environment());
            contract.set_address(&maybe_module_addr[0].1);
            let module: T = contract.into();
            Ok(module)
        } else {
            Err(AbstractClientError::ModuleNotInstalled {})
        }
    }

    fn missing_modules(
        &self,
        modules_to_maybe_install: &[ModuleInstallConfig],
    ) -> AbstractClientResult<Vec<ModuleInstallConfig>> {
        // TODO: Is it something supposed to work? Do we just create new sub account with those modules or what?
        if self.install_on_sub_account {
            return Ok(vec![]);
        }

        let mut modules_to_install = vec![];
        let installed_modules = self.module_infos()?.module_infos;
        for module in modules_to_maybe_install {
            match installed_modules
                .iter()
                .find(|m| m.id == module.module.id())
            {
                Some(_installed_module) => {
                    // Have this module installed, do we want to try to upgrade module if version is different?
                }
                None => modules_to_install.push(module.clone()),
            }
        }

        Ok(modules_to_install)
    }

    /// Claim a namespace for an existing account
    pub fn claim_namespace(
        &self,
        namespace: impl Into<String>,
    ) -> Result<Chain::Response, AbstractInterfaceError> {
        self.abstr_account.claim_namespace(namespace)
    }
}

impl<Chain: MutCwEnv> Account<Chain> {
    /// Set balance for the Proxy
    pub fn set_balance(&self, amount: &[Coin]) -> AbstractClientResult<()> {
        self.environment()
            .set_balance(&self.address()?, amount.to_vec())
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Add balance to the Proxy
    pub fn add_balance(&self, amount: &[Coin]) -> AbstractClientResult<()> {
        self.environment()
            .add_balance(&self.address()?, amount.to_vec())
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

impl<Chain: CwEnv> Display for Account<Chain> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abstr_account)
    }
}

impl<Chain: CwEnv> Debug for Account<Chain> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
        // TODO:
        // write!(f, "{:?}", self.abstr_account)
    }
}

#[cfg(test)]
pub mod test {
    use abstract_interface::{Abstract, VCQueryFns};
    use abstract_std::objects::namespace::Namespace;
    use cw_orch::{contract::Deploy, mock::MockBech32};

    use crate::AbstractClient;

    #[test]
    fn namespace_after_creation() -> cw_orch::anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let abstr = AbstractClient::builder(mock.clone()).build()?;

        let my_namespace = "my-namespace";
        let new_account = abstr_builder().build()?;
        new_account.claim_namespace(my_namespace)?;

        // Verify the namespace exists
        let abstr = Abstract::load_from(mock.clone())?;
        let namespace_response = abstr
            .version_control
            .namespace(Namespace::new(my_namespace)?)?;

        match namespace_response {
            abstract_std::version_control::NamespaceResponse::Claimed(c) => {
                assert_eq!(c.account_id, new_account.id()?)
            }
            abstract_std::version_control::NamespaceResponse::Unclaimed {} => {
                panic!("Expected claimed namespace")
            }
        }

        Ok(())
    }
}
