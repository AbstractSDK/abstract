//! # Represents Abstract Publisher
//!
//! [`Publisher`] is an Account with helpers for publishing and maintaining Abstract Applications and Adapters

use abstract_interface::{
    AdapterDeployer, AppDeployer, DeployStrategy, RegisteredModule, StandaloneDeployer,
};
use abstract_std::objects::{gov_type::GovernanceDetails, namespace::Namespace, AssetEntry};
use cw_orch::{
    contract::Contract,
    prelude::{ContractInstance, CwEnv},
};
use serde::Serialize;

use crate::{
    account::{Account, AccountBuilder},
    client::AbstractClientResult,
    Environment,
};

/// A builder for creating [`Publishers`](Account).
/// Get the builder from the [`AbstractClient::publisher_builder`](crate::AbstractClient)
/// and create the account with the `build` method.
///
/// ```
/// # use abstract_client::{AbstractClientError, Environment};
/// # use cw_orch::prelude::*;
/// # let chain = MockBech32::new("mock");
/// # let abstr_client = abstract_client::AbstractClient::builder(chain).build().unwrap();
/// # let chain = abstr_client.environment();
/// use abstract_client::{AbstractClient, Publisher, Namespace};
///
/// let client = AbstractClient::new(chain)?;
///
/// let namespace = Namespace::new("alice-namespace")?;
/// let publisher: Publisher<MockBech32> = client.publisher_builder(namespace)
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
        namespace: Namespace,
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

    /// Overwrite the configured namespace
    pub fn namespace(&mut self, namespace: Namespace) -> &mut Self {
        self.account_builder.namespace(namespace);
        self
    }

    /// Governance of the account.
    /// Defaults to the [`GovernanceDetails::Monarchy`] variant, owned by the sender
    pub fn ownership(&mut self, ownership: GovernanceDetails<String>) -> &mut Self {
        self.account_builder.ownership(ownership);
        self
    }

    /// Install modules on a new sub-account instead of current account.
    /// Defaults to `true`
    pub fn install_on_sub_account(&mut self, value: bool) -> &mut Self {
        self.account_builder.install_on_sub_account(value);
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

    /// Publish an Abstract Standalone
    pub fn publish_standalone<
        M: ContractInstance<Chain>
            + RegisteredModule
            + From<Contract<Chain>>
            + StandaloneDeployer<Chain>,
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
    ) -> AbstractClientResult<M> {
        let contract = Contract::new(M::module_id().to_owned(), self.account.environment());
        let adapter: M = contract.into();
        adapter.deploy(M::module_version().parse()?, init_msg, DeployStrategy::Try)?;
        Ok(adapter)
    }

    /// Abstract Account of the publisher
    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }
}
