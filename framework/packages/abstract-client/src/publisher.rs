use abstract_core::{
    manager::{ModuleAddressesResponse, ModuleInfosResponse},
    objects::{gov_type::GovernanceDetails, AssetEntry},
};
use abstract_interface::{
    AdapterDeployer, AppDeployer, DependencyCreation, DeployStrategy, InstallConfig,
    RegisteredModule,
};
use cosmwasm_std::{Addr, Coin};
use cw_orch::{
    contract::Contract,
    prelude::{ContractInstance, CwEnv},
};
use serde::Serialize;

use crate::{
    account::{Account, AccountBuilder},
    application::Application,
    client::AbstractClientResult,
    infrastructure::Environment,
};

/// PublisherBuilder is a builder for creating publisher account.
/// It's intended to be used from [`crate::client::AbstractClient::publisher_builder`]
/// and created with method `build`
///
/// ```
/// # use abstract_client::{__doc_setup_mock, error::AbstractClientError, infrastructure::Environment};
/// # let abstr_client = __doc_setup_mock!();
/// # let chain = abstr_client.environment();
/// use abstract_client::client::AbstractClient;
///
/// let client = AbstractClient::new(chain)?;
/// let account = client.publisher_builder().name("alice").build()?;
/// # Ok::<(), AbstractClientError>(())
/// ```
pub struct PublisherBuilder<'a, Chain: CwEnv> {
    account_builder: AccountBuilder<'a, Chain>,
}

impl<'a, Chain: CwEnv> PublisherBuilder<'a, Chain> {
    pub(crate) fn new(
        mut account_builder: AccountBuilder<'a, Chain>,
        namespace: impl Into<String>,
    ) -> Self {
        account_builder.namespace(namespace);
        Self { account_builder }
    }

    /// Username for the account
    /// Defaults to "Default Abstract Account"
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.account_builder.name(name);
        self
    }

    /// Description for the account
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.account_builder.description(description);
        self
    }

    /// http(s) or ipfs link for the account
    pub fn link(&mut self, link: impl Into<String>) -> &mut Self {
        self.account_builder.link(link);
        self
    }

    /// Base Asset for the account
    pub fn base_asset(&mut self, base_asset: AssetEntry) -> &mut Self {
        self.account_builder.base_asset(base_asset);
        self
    }

    /// Governance of the account.
    /// Defaults to the Monarchy, owned by the sender
    pub fn governance_details(
        &mut self,
        governance_details: GovernanceDetails<String>,
    ) -> &mut Self {
        self.account_builder.governance_details(governance_details);
        self
    }

    pub fn build(&self) -> AbstractClientResult<Publisher<Chain>> {
        let account = self.account_builder.build()?;
        Ok(Publisher { account })
    }
}

/// A publisher represents an account that owns a namespace with the goal of publishing software to the module-store.
pub struct Publisher<Chain: CwEnv> {
    account: Account<Chain>,
}

impl<Chain: CwEnv> Publisher<Chain> {
    pub(crate) fn new(account: Account<Chain>) -> Self {
        Self { account }
    }

    /// Install an application
    /// creates a new sub-account and installs the application on it.
    pub fn install_app<
        M: ContractInstance<Chain> + InstallConfig + From<Contract<Chain>> + Clone,
    >(
        &self,
        configuration: &M::InitMsg,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        self.account.install_app(configuration, funds)
    }

    /// Install application with it's dependencies with provided dependencies config
    /// creates a new sub-account and installs the application on it.
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
        self.account
            .install_app_with_dependencies(module_configuration, dependencies_config, funds)
    }

    /// Publish Abstract App
    pub fn publish_app<
        M: ContractInstance<Chain> + RegisteredModule + From<Contract<Chain>> + AppDeployer<Chain>,
    >(
        &self,
    ) -> AbstractClientResult<()> {
        let contract = Contract::new(M::module_id().to_owned(), self.account.environment());
        let app: M = contract.into();
        app.deploy(M::module_version().parse()?, DeployStrategy::Try)
            .map_err(Into::into)
    }

    /// Publish Abstract Adapter
    // TODO: why it's publish app and deploy adapter, shouldn't we stick to one way?
    pub fn deploy_adapter<
        CustomInitMsg: Serialize,
        M: ContractInstance<Chain>
            + RegisteredModule
            + From<Contract<Chain>>
            + AdapterDeployer<Chain, CustomInitMsg>,
    >(
        &self,
        init_msg: CustomInitMsg,
    ) -> AbstractClientResult<()> {
        let contract = Contract::new(M::module_id().to_owned(), self.account.environment());
        let app: M = contract.into();
        app.deploy(M::module_version().parse()?, init_msg, DeployStrategy::Try)
            .map_err(Into::into)
    }

    /// Abstract Account of the publisher
    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }

    // TODO: add `account_admin` fn to get the (Sub-)Account's admin.
    // TODO: I don't get why it's called admin
    /// Address of the manager
    pub fn admin(&self) -> AbstractClientResult<Addr> {
        self.account.manager()
    }

    /// Address of the proxy
    pub fn proxy(&self) -> AbstractClientResult<Addr> {
        self.account.proxy()
    }

    /// Module infos of installed modules on account
    pub fn module_infos(&self) -> AbstractClientResult<ModuleInfosResponse> {
        self.account.module_infos()
    }

    /// Addresses of installed modules on account
    pub fn module_addresses(
        &self,
        ids: Vec<String>,
    ) -> AbstractClientResult<ModuleAddressesResponse> {
        self.account.module_addresses(ids)
    }
}
