//! # Represents Abstract Account
//!
//! [`Account`] allows you to interact with your or another user Abstract Account
//!
//! Example of creating an account
//! ```
//! # use abstract_client::error::AbstractClientError;
//! use abstract_client::{client::AbstractClient, account::Account};
//! use cw_orch::prelude::Mock;
//!
//! # let client: AbstractClient<Mock> = AbstractClient::builder("sender").build()?;
//!
//! let alice_account: Account<Mock> = client
//!     .account_builder()
//!     .name("Alice")
//!     .build()?;
//!
//! assert_eq!(alice_account.owner()?, client.sender());
//! # Ok::<(), AbstractClientError>(())
//! ```
use abstract_core::{
    manager::{
        state::AccountInfo, InfoResponse, ManagerModuleInfo, ModuleAddressesResponse,
        ModuleInfosResponse, ModuleInstallConfig,
    },
    objects::{
        gov_type::GovernanceDetails, module, namespace::Namespace,
        nested_admin::MAX_ADMIN_RECURSION, validation::verifiers, AccountId, AssetEntry,
    },
    version_control::NamespaceResponse,
    PROXY,
};
use abstract_interface::{
    Abstract, AbstractAccount, AbstractInterfaceError, AccountDetails, DependencyCreation,
    InstallConfig, ManagerExecFns, ManagerQueryFns, RegisteredModule, VCQueryFns,
};

use cosmwasm_std::{to_json_binary, Attribute, CosmosMsg, Empty, Uint128};
use cw_orch::prelude::*;
use cw_orch::{contract::Contract, environment::MutCwEnv};

use crate::{
    application::Application,
    client::AbstractClientResult,
    error::AbstractClientError,
    infrastructure::{Environment, Infrastructure},
};

/// A builder for creating [`Accounts`](Account).
/// Get the builder from the [`AbstractClient::account_builder`](crate::client::AbstractClient)
/// and create the account with the `build` method.
///
/// ```
/// # use abstract_client::{error::AbstractClientError, infrastructure::Environment};
/// # let abstr_client = abstract_client::client::AbstractClient::builder("sender").build().unwrap();
/// # let chain = abstr_client.environment();
/// use abstract_client::client::AbstractClient;
///
/// let client = AbstractClient::new(chain)?;
/// let account: Account<Mock> = client.account_builder()
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
    base_asset: Option<AssetEntry>,
    // TODO: Decide if we want to abstract this as well.
    ownership: Option<GovernanceDetails<String>>,
    // TODO: How to handle install_modules?
    fetch_if_namespace_claimed: bool,
}

impl<'a, Chain: CwEnv> AccountBuilder<'a, Chain> {
    pub(crate) fn new(abstr: &'a Abstract<Chain>) -> Self {
        Self {
            abstr,
            name: None,
            description: None,
            link: None,
            namespace: None,
            base_asset: None,
            ownership: None,
            fetch_if_namespace_claimed: false,
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

    /// Base Asset for the account
    pub fn base_asset(&mut self, base_asset: AssetEntry) -> &mut Self {
        self.base_asset = Some(base_asset);
        self
    }

    /// Try to fetch the account before creating it if the namespace is already claimed.
    pub fn fetch_if_namespace_claimed(&mut self, value: bool) -> &mut Self {
        self.fetch_if_namespace_claimed = value;
        self
    }

    /// Governance of the account.
    /// Defaults to the [`GovernanceDetails::Monarchy`] variant, owned by the sender
    pub fn ownership(&mut self, ownership: GovernanceDetails<String>) -> &mut Self {
        self.ownership = Some(ownership);
        self
    }

    /// Builds the [`Account`].
    pub fn build(&self) -> AbstractClientResult<Account<Chain>> {
        if self.fetch_if_namespace_claimed {
            // Check if namespace already claimed
            if let Some(ref namespace) = self.namespace {
                let account_from_namespace_result: Option<Account<Chain>> =
                    Account::from_namespace(self.abstr, namespace.clone())?;

                // Only return if the account can be retrieved without errors.
                if let Some(account_from_namespace) = account_from_namespace_result {
                    return Ok(account_from_namespace);
                }
            }
        }

        let chain = self.abstr.version_control.get_chain();
        let sender = chain.sender().to_string();
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
        if let Some(namespace) = &self.namespace {
            module::validate_name(namespace)?;
        }

        let abstract_account = self.abstr.account_factory.create_new_account(
            AccountDetails {
                name,
                description: self.description.clone(),
                link: self.link.clone(),
                namespace: self.namespace.as_ref().map(ToString::to_string),
                base_asset: self.base_asset.clone(),
                install_modules: vec![],
            },
            ownership,
            Some(&[]),
        )?;
        Ok(Account::new(abstract_account))
    }
}

/// Represents an existing Abstract account.
///
/// Get this struct from [`AbstractClient::account_from_namespace`](crate::client::AbstractClient)
/// or create a new account with the [`AccountBuilder`].
#[derive(Clone)]
pub struct Account<Chain: CwEnv> {
    pub(crate) abstr_account: AbstractAccount<Chain>,
}

struct ParsedAccountCreationResponse {
    sub_account_id: u32,
    module_address: String,
}

impl<Chain: CwEnv> Account<Chain> {
    pub(crate) fn new(abstract_account: AbstractAccount<Chain>) -> Self {
        Self {
            abstr_account: abstract_account,
        }
    }

    pub(crate) fn from_namespace(
        abstr: &Abstract<Chain>,
        namespace: Namespace,
    ) -> AbstractClientResult<Option<Self>> {
        // TODO: Update `Namespace` query to return Option.
        // https://linear.app/abstract-sdk/issue/ABS-288/fix-the-vc-namespace-query
        let namespace_response: Option<NamespaceResponse> =
            abstr.version_control.namespace(namespace).ok();
        Ok(namespace_response
            .map(|resp| AbstractAccount::new(abstr, resp.account_id))
            .map(Self::new))
    }

    /// Get the [`AccountId`] of the Account
    pub fn id(&self) -> AbstractClientResult<AccountId> {
        self.abstr_account.id().map_err(Into::into)
    }

    /// Get the [`AccountId`] of the Account
    pub fn id(&self) -> AbstractClientResult<AccountId> {
        self.abstr_account.id().map_err(Into::into)
    }

    /// Query account balance of a given denom
    // TODO: Asset balance?
    pub fn query_balance(&self, denom: impl Into<String>) -> AbstractClientResult<Uint128> {
        let coins = self
            .environment()
            .balance(self.proxy()?, Some(denom.into()))
            .map_err(Into::into)?;

        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    /// Query account balances of all denoms
    pub fn query_balances(&self) -> AbstractClientResult<Vec<Coin>> {
        self.environment()
            .balance(self.proxy()?, None)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Query account info
    pub fn info(&self) -> AbstractClientResult<AccountInfo<Addr>> {
        let info_response: InfoResponse = self.abstr_account.manager.info()?;
        Ok(info_response.info)
    }

    /// Install an application on the account.
    /// This creates a new sub-account and installs the application on it.
    pub fn install_app<M: ContractInstance<Chain> + InstallConfig + From<Contract<Chain>>>(
        &self,
        configuration: &M::InitMsg,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        self.install_app_internal(vec![M::install_config(configuration)?], funds)
    }

    /// Install an application on current account.
    pub fn install_adapter<
        M: ContractInstance<Chain> + InstallConfig<InitMsg = Empty> + From<Contract<Chain>>,
    >(
        &self,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let install_adapter_response = self
            .abstr_account
            .manager
            .install_modules(vec![M::install_config(&Empty {})?], Some(funds))?;

        let adapter_addr = Self::parse_adapter_installing_response(install_adapter_response);
        let contract = Contract::new(M::module_id().to_owned(), self.environment())
            .with_address(Some(&adapter_addr));

        let adapter: M = contract.into();

        Application::new(Account::new(self.abstr_account.clone()), adapter)
    }

    /// Creates a new sub-account on the current account and
    /// installs an App module and its dependencies with the provided dependencies config. \
    ///
    /// The returned [`Application`] is a wrapper around the sub-account and simplifies interaction with the App module.
    pub fn install_app_with_dependencies<
        M: ContractInstance<Chain>
            + DependencyCreation
            + InstallConfig
            + From<Contract<Chain>>
            + Clone,
    >(
        &self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let mut install_configs: Vec<ModuleInstallConfig> =
            M::dependency_install_configs(dependencies_config)?;
        install_configs.push(M::install_config(module_configuration)?);

        self.install_app_internal(install_configs, funds)
    }

    /// Returns owner of the account
    pub fn ownership(&self) -> AbstractClientResult<cw_ownable::Ownership<String>> {
        self.abstr_account.manager.ownership().map_err(Into::into)
    }

    /// Returns the owner address of the account.
    /// If the account is a sub-account, it will return the top-level owner address.
    pub fn owner(&self) -> AbstractClientResult<Addr> {
        let mut governance = self.abstr_account.manager.info()?.info.governance_details;

        let environment = self.environment();
        // Get sub-accounts until we get non-sub-account governance or reach recursion limit
        for _ in 0..MAX_ADMIN_RECURSION {
            match &governance {
                GovernanceDetails::SubAccount { manager, .. } => {
                    governance = environment
                        .query::<_, InfoResponse>(
                            &abstract_core::manager::QueryMsg::Info {},
                            manager,
                        )
                        .map_err(|err| err.into())?
                        .info
                        .governance_details;
                }
                _ => break,
            }
        }

        // Get top level account owner address
        governance
            .owner_address()
            .ok_or(AbstractClientError::RenouncedAccount {})
    }

    /// Executes a [`CosmosMsg`] on the proxy of the account.
    pub fn execute(
        &self,
        execute_msgs: impl IntoIterator<Item = impl Into<CosmosMsg>>,
        funds: &[Coin],
    ) -> AbstractClientResult<<Chain as TxHandler>::Response> {
        let msgs = execute_msgs.into_iter().map(Into::into).collect();
        self.abstr_account
            .manager
            .execute(
                &abstract_core::manager::ExecuteMsg::ExecOnModule {
                    module_id: PROXY.to_owned(),
                    exec_msg: to_json_binary(&abstract_core::proxy::ExecuteMsg::ModuleAction {
                        msgs,
                    })
                    .map_err(AbstractInterfaceError::from)?,
                },
                Some(funds),
            )
            .map_err(Into::into)
    }

    /// Module infos of installed modules on account
    pub fn module_infos(&self) -> AbstractClientResult<ModuleInfosResponse> {
        let mut module_infos: Vec<ManagerModuleInfo> = vec![];
        loop {
            let last_module_id: Option<String> = module_infos
                .last()
                .map(|module_info| module_info.id.clone());
            let res: ModuleInfosResponse = self
                .abstr_account
                .manager
                .module_infos(None, last_module_id)?;
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
        self.abstr_account
            .manager
            .module_addresses(ids)
            .map_err(Into::into)
    }

    /// Address of the proxy
    pub fn proxy(&self) -> AbstractClientResult<Addr> {
        self.abstr_account.proxy.address().map_err(Into::into)
    }

    /// Address of the manager
    pub fn manager(&self) -> AbstractClientResult<Addr> {
        self.abstr_account.manager.address().map_err(Into::into)
    }

    fn install_app_internal<
        M: ContractInstance<Chain> + RegisteredModule + From<Contract<Chain>>,
    >(
        &self,
        modules: Vec<ModuleInstallConfig>,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        // Create sub account.
        let sub_account_response = self.abstr_account.manager.create_sub_account(
            modules,
            "Sub Account".to_owned(),
            None,
            None,
            None,
            None,
            funds,
        )?;

        let parsed_account_creation_response =
            Self::parse_account_creation_response(sub_account_response);

        let sub_account: AbstractAccount<Chain> = AbstractAccount::new(
            &self.infrastructure()?,
            AccountId::local(parsed_account_creation_response.sub_account_id),
        );

        let contract =
            Contract::new(M::module_id().to_owned(), self.environment()).with_address(Some(
                &Addr::unchecked(parsed_account_creation_response.module_address),
            ));

        let app: M = contract.into();

        Application::new(Account::new(sub_account), app)
    }

    fn parse_account_creation_response(
        response: <Chain as TxHandler>::Response,
    ) -> ParsedAccountCreationResponse {
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

    fn parse_adapter_installing_response(response: <Chain as TxHandler>::Response) -> Addr {
        let wasm_abstract_attributes: Vec<Attribute> = response
            .events()
            .into_iter()
            .filter(|e| e.ty == "wasm-abstract")
            .flat_map(|e| e.attributes)
            .collect();

        let module_addresses: Option<String> = wasm_abstract_attributes
            .iter()
            .find(|a| a.key == "new_modules")
            .map(|a| a.value.parse().unwrap());

        // We install only one adapter
        Addr::unchecked(module_addresses.unwrap())
    }
}

impl<Chain: MutCwEnv> Account<Chain> {
    /// Set balance for the Proxy
    pub fn set_balance(&self, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .set_balance(&self.proxy()?, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Add balance to the Proxy
    pub fn add_balance(&self, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .add_balance(&self.proxy()?, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }
}
