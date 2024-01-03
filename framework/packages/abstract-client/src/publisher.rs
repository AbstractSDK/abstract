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
