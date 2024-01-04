use abstract_core::objects::{gov_type::GovernanceDetails, AssetEntry};
use abstract_interface::{AdapterDeployer, AppDeployer, DeployStrategy, RegisteredModule};
use cw_orch::{
    contract::Contract,
    prelude::{ContractInstance, CwEnv},
};
use serde::Serialize;

use crate::{
    account::{Account, AccountBuilder},
    client::AbstractClientResult,
    infrastructure::Environment,
};

/// A builder for creating [`Publishers`](Account).
/// Get the builder from the [`AbstractClient::publisher_builder`](crate::client::AbstractClient)
/// and create the account with the `build` method.
///
/// ```
/// # use abstract_client::{error::AbstractClientError, infrastructure::Environment};
/// # let abstr_client = abstract_client::client::AbstractClient::builder("sender").build().unwrap();
/// # let chain = abstr_client.environment();
/// use abstract_client::client::AbstractClient;
///
/// let client = AbstractClient::new(chain)?;
/// let publisher: Publisher<Mock> = client.publisher_builder("alice-namespace")
///     .name("alice")
///     // other configurations
///     .build()?;
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
        account_builder.fetch_if_namespace_claimed(true);
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
    /// Defaults to the [`GovernanceDetails::Monarchy`] variant, owned by the sender
    pub fn ownership(&mut self, ownership: GovernanceDetails<String>) -> &mut Self {
        self.account_builder.ownership(ownership);
        self
    }

    /// Builds the [`Publisher`].
    /// Creates an account if the namespace is not already owned.
    pub fn build(&self) -> AbstractClientResult<Publisher<Chain>> {
        let account = self.account_builder.build()?;
        Ok(Publisher { account })
    }
}

/// A Publisher represents an account that owns a namespace with the goal of publishing modules to the on-chain module-store.
pub struct Publisher<Chain: CwEnv> {
    account: Account<Chain>,
}

impl<Chain: CwEnv> Publisher<Chain> {
    pub(crate) fn new(account: Account<Chain>) -> Self {
        Self { account }
    }

    /// Publish an Abstract App
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

    /// Publish an Abstract Adapter
    pub fn publish_adapter<
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
        let adapter: M = contract.into();
        adapter
            .deploy(M::module_version().parse()?, init_msg, DeployStrategy::Try)
            .map_err(Into::into)
    }

    /// Abstract Account of the publisher
    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }
}
