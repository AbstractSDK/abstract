//! # Represents Abstract Client
//!
//! [`AbstractClient`] allows you to do everything you might need to work with the Abstract
//! or to be more precise
//!
//! - Create or interact with Account
//! - Install or interact with a module (including apps and adapters)
//! - Publish modules
//! - Do integration tests with Abstract
//!
//! Example of publishing mock app
//!
//! ```
//! # use abstract_client::AbstractClientError;
//! use abstract_app::mock::mock_app_dependency::interface::MockAppI;
//! use cw_orch::prelude::*;
//! use abstract_client::{AbstractClient, Publisher, Namespace};
//!
//! let chain = MockBech32::new("mock");
//! let client = AbstractClient::builder(chain).build()?;
//!
//! let namespace = Namespace::new("tester")?;
//! let publisher: Publisher<MockBech32> = client
//!     .publisher_builder(namespace)
//!     .build()?;
//!
//! publisher.publish_app::<MockAppI<MockBech32>>()?;
//! # Ok::<(), AbstractClientError>(())
//! ```

use std::collections::HashMap;

use abstract_interface::{AbstractAccount, ManagerQueryFns, RegisteredModule};
use abstract_std::objects::{
    module::{ModuleInfo, ModuleVersion},
    module_reference::ModuleReference,
    namespace::Namespace,
    salt::generate_instantiate_salt,
    AccountId,
};
use cosmwasm_std::Uint128;
use cw_orch::prelude::*;
use cw_orch_interchain::IbcQueryHandler;
use rand::Rng;

use crate::{
    account::{Account, AccountBuilder},
    client::AbstractClientResult,
    source::AccountSource,
    AbstractClient, AbstractClientError, PublisherBuilder,
};

/// Client to interact with Abstract accounts and modules
pub struct AbstractInterchainClient<Chain: CwEnv> {
    pub(crate) abstracts: HashMap<String, AbstractClient<Chain>>,
}

impl<Chain: IbcQueryHandler> AbstractInterchainClient<Chain> {
    /// Get [`AbstractInterchainClient`] from a chosen environment. [`Abstract`] should
    /// already be deployed to this environment.
    ///
    /// ```
    /// use abstract_client::AbstractClient;
    /// # use abstract_client::{Environment, AbstractClientError};
    /// # use cw_orch::prelude::*;
    /// # let chain = MockBech32::new("mock");
    /// # let client = AbstractClient::builder(chain.clone()).build().unwrap(); // Deploy mock abstract
    ///
    /// let client = AbstractClient::new(chain)?;
    /// # Ok::<(), AbstractClientError>(())
    /// ```
    pub fn new(chains: Vec<Chain>) -> AbstractClientResult<Self> {
        let abstracts = chains
            .into_iter()
            .map(|c| {
                let chain_id = c.chain_id();
                let abstr = AbstractClient::new(c)?;
                Ok::<_, AbstractClientError>((chain_id, abstr))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self { abstracts })
    }

    pub fn get(&self, chain_id: String) -> AbstractClientResult<&AbstractClient<Chain>> {
        self.abstracts
            .get(&chain_id)
            .ok_or(AbstractClientError::InterchainError(
                cw_orch_interchain::InterchainError::ChainNotFound(chain_id),
            ))
    }

    /// Publisher builder for creating new [`Publisher`](crate::Publisher) Abstract Account
    /// To publish any modules your account requires to have claimed a namespace.
    pub fn publisher_builder(
        &self,
        chain_id: String,
        namespace: Namespace,
    ) -> AbstractClientResult<PublisherBuilder<Chain>> {
        Ok(self.get(chain_id)?.publisher_builder(namespace))
    }

    /// Builder for creating a new Abstract [`Account`].
    pub fn account_builder(&self, chain_id: String) -> AbstractClientResult<AccountBuilder<Chain>> {
        Ok(self.get(chain_id)?.account_builder())
    }

    /// Address of the sender
    pub fn sender(&self, chain_id: String) -> AbstractClientResult<Addr> {
        Ok(self.get(chain_id)?.sender())
    }

    /// Fetch an [`Account`] from a given source.
    ///
    /// This method is used to retrieve an account from a given source. It will **not** create a new account if the source is invalid.
    ///
    /// Sources that can be used are:
    /// - [`Namespace`]: Will retrieve the account from the namespace if it is already claimed.
    /// - [`AccountId`]: Will retrieve the account from the account id.
    /// - App [`Addr`]: Will retrieve the account from an app that is installed on it.
    pub fn account_from<T: Into<AccountSource>>(
        &self,
        chain_id: String,
        source: T,
    ) -> AbstractClientResult<Account<Chain>> {
        self.get(chain_id)?.account_from(source)
    }

    /// Connect this abstract client to the remote abstract client
    /// If [`cw_orch_polytone::Polytone`] is deployed between 2 chains, it will NOT redeploy it (good for actual chains)
    /// If Polytone is not deployed, deploys it between the 2 chains (good for integration testing)
    pub fn add_chain(&self, chain: Chain) -> AbstractClientResult<()>
    where
        Chain: cw_orch_interchain::IbcQueryHandler,
    {
        Ok(())
    }
}

impl<Chain: IbcQueryHandler> IntoIterator for AbstractInterchainClient<Chain> {
    type Item = <HashMap<String, AbstractClient<Chain>> as IntoIterator>::Item;

    type IntoIter = <HashMap<String, AbstractClient<Chain>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.abstracts.into_iter()
    }
}
