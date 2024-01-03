//! # Represents Abstract Client
//!
//! [`AbstractClient`] allows you to do everything you might need to work with the abstract
//! or to be more precise
//! - Create or interact with Account
//! - Install or interact with a module (including apps and adapters)
//! - Publish modules
//! - Do integration tests with Abstract
//!
//! Example of creating account and installing dex adapter
//! ```no_run
//! use cw_orch::prelude::*;
//! use cw_orch::daemon::{networks::parse_network, Daemon};
//! use abstract_client::client::AbstractClient;
//! use abstract_dex_adapter::interface::DexAdapter;
//! use tokio::runtime::Runtime;
//!
//! let rt = Runtime::new()?;
//! let juno_testnet = Daemon::builder()
//!         .handle(rt.handle())
//!         .chain(parse_network("uni-6").unwrap())
//!         // Your mnemonic
//!         .mnemonic(TEST_MNEMONIC)
//!         .build()
//!         .unwrap();
//! let client = AbstractClient::new(juno_testnet)?;
//! let account: Account<Daemon> = client.account_builder().name("Alice").build()?;
//! account.install_app::<DexAdapter<Daemon>>(&cosmwasm_std::Empty{ }, &[])?;
//!
//! # Ok::<(), AbstractClientError>(())
//! ```

use abstract_interface::{Abstract, VersionControl};
use cosmwasm_std::{Addr, BlockInfo, Coin, Uint128};
use cw_orch::{deploy::Deploy, environment::MutCwEnv, prelude::CwEnv};

use crate::{
    account::{Account, AccountBuilder},
    error::AbstractClientError,
    infrastructure::Environment,
    publisher::{Publisher, PublisherBuilder},
};

/// Client to interact with Abstract accounts and modules
pub struct AbstractClient<Chain: CwEnv> {
    pub(crate) abstr: Abstract<Chain>,
}

/// The result type for the Abstract Client.
pub type AbstractClientResult<T> = Result<T, AbstractClientError>;

impl<Chain: CwEnv> AbstractClient<Chain> {
    /// Get abstract client from a chosen network. Abstract should be
    /// already deployed on this chain
    ///
    /// ```
    /// # use abstract_client::error::AbstractClientError;
    /// # let sender = cosmwasm_std::Addr::unchecked("sender");
    /// # let chain = cw_orch::prelude::Mock::new(&sender);
    /// use abstract_client::client::AbstractClient;
    ///
    /// let client = AbstractClient::new(chain)?;
    /// # Ok::<(), AbstractClientError>(())
    /// ```
    pub fn new(chain: Chain) -> AbstractClientResult<Self> {
        // TODO: New error type for not found abstract on this chain?
        let abstr = Abstract::load_from(chain)?;
        Ok(Self { abstr })
    }

    // TODO: No user friendly API for AnsHost
    // pub fn name_service(&self) -> &AnsHost<Chain> {
    //     &self.abstr.ans_host
    // }

    /// Version Control contract API
    /// ```
    /// # use abstract_client::__doc_setup_mock;
    /// # use abstract_client::error::AbstractClientError;
    /// # let client = __doc_setup_mock!();
    /// use abstract_core::objects::{module_reference::ModuleReference, module::ModuleInfo};
    /// // For getting version control address
    /// use cw_orch::prelude::*;
    ///
    /// let version_control = client.version_control();
    /// let vc_module = version_control.module(ModuleInfo::from_id_latest("abstract:version-control")?)?;
    /// assert_eq!(vc_module.reference, ModuleReference::Native(version_control.address()?));
    /// # Ok::<(), AbstractClientError>(())
    /// ```
    pub fn version_control(&self) -> &VersionControl<Chain> {
        &self.abstr.version_control
    }

    /// Return current block info see [`BlockInfo`].
    pub fn block_info(&self) -> AbstractClientResult<BlockInfo> {
        self.environment()
            .block_info()
            .map_err(|e| AbstractClientError::CwOrch(e.into()))
    }

    /// Retrieve [`Publisher`] that holds this namespace
    pub fn publisher_from_namespace(
        &self,
        namespace: &str,
    ) -> AbstractClientResult<Publisher<Chain>> {
        Ok(Publisher::new(self.account_from_namespace(namespace)?))
    }

    /// Publisher builder for creating new [`Publisher`] Abstract Account
    /// To publish any modules your account requires to have namespace
    pub fn publisher_builder(&self, namespace: &str) -> PublisherBuilder<Chain> {
        PublisherBuilder::new(AccountBuilder::new(&self.abstr), namespace)
    }

    /// Publisher builder for creating a new Abstract Account
    pub fn account_builder(&self) -> AccountBuilder<Chain> {
        AccountBuilder::new(&self.abstr)
    }

    /// Retrieve Abstract [`Account`] that holds this namespace
    pub fn account_from_namespace(&self, namespace: &str) -> AbstractClientResult<Account<Chain>> {
        Account::from_namespace(&self.abstr, namespace)
    }

    /// Address of the sender
    pub fn sender(&self) -> Addr {
        self.environment().sender()
    }

    /// Retrieve denom balance for chosen address
    pub fn query_balance(
        &self,
        address: &Addr,
        denom: impl Into<String>,
    ) -> AbstractClientResult<Uint128> {
        let coins = self
            .environment()
            .balance(address, Some(denom.into()))
            .map_err(Into::into)?;
        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    /// Retrieve balances of all denoms for chosen address
    pub fn query_balances(&self, address: &Addr) -> AbstractClientResult<Vec<Coin>> {
        self.environment()
            .balance(address, None)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Waits for a specified number of blocks.
    pub fn wait_blocks(&self, amount: u64) -> AbstractClientResult<()> {
        self.environment()
            .wait_blocks(amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Waits for a specified number of blocks.
    pub fn wait_seconds(&self, secs: u64) -> AbstractClientResult<()> {
        self.environment()
            .wait_seconds(secs)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Waits for next block.
    pub fn next_block(&self) -> AbstractClientResult<()> {
        self.environment()
            .next_block()
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

impl<Chain: MutCwEnv> AbstractClient<Chain> {
    /// Set balance for an address
    pub fn set_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .set_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Add balance for the address
    pub fn add_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .add_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }
}
