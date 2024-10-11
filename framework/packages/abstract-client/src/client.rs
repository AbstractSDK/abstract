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
//! use abstract_testing::prelude::*;
//!
//! let chain = MockBech32::new("mock");
//! let client = AbstractClient::builder(chain.clone()).build_mock()?;
//!
//! let namespace = Namespace::new("tester")?;
//! let publisher: Publisher<MockBech32> = client
//!     .publisher_builder(namespace)
//!     .build()?;
//!
//! publisher.publish_app::<MockAppI<MockBech32>>()?;
//! # Ok::<(), AbstractClientError>(())
//! ```

use abstract_interface::{
    Abstract, AccountI, AnsHost, IbcClient, ModuleFactory, RegisteredModule, Registry,
    RegistryQueryFns,
};
use abstract_std::objects::{
    module::{ModuleInfo, ModuleStatus, ModuleVersion},
    module_reference::ModuleReference,
    namespace::Namespace,
    salt::generate_instantiate_salt,
    AccountId,
};
use cosmwasm_std::{BlockInfo, Uint128};
use cw_orch::{contract::Contract, environment::Environment as _, prelude::*};
use rand::Rng;

use crate::{
    account::{Account, AccountBuilder},
    source::AccountSource,
    AbstractClientError, Environment, PublisherBuilder, Service,
};

/// Client to interact with Abstract accounts and modules
#[derive(Clone)]
pub struct AbstractClient<Chain: CwEnv> {
    pub(crate) abstr: Abstract<Chain>,
}

/// The result type for the Abstract Client.
pub type AbstractClientResult<T> = Result<T, AbstractClientError>;

impl<Chain: CwEnv> AbstractClient<Chain> {
    /// Get [`AbstractClient`] from a chosen environment. [`Abstract`] should
    /// already be deployed to this environment.
    ///
    /// ```
    /// use abstract_client::AbstractClient;
    /// # use abstract_client::{Environment, AbstractClientError};
    /// # use cw_orch::prelude::*;
    /// # let chain = MockBech32::new("mock");
    /// # let client = AbstractClient::builder(chain.clone()).build_mock().unwrap(); // Deploy mock abstract
    ///
    /// let client = AbstractClient::new(chain)?;
    /// # Ok::<(), AbstractClientError>(())
    /// ```
    pub fn new(chain: Chain) -> AbstractClientResult<Self> {
        let abstr = Abstract::load_from(chain)?;
        Ok(Self { abstr })
    }

    /// Version Control contract API
    ///
    /// The Version Control contract is a database contract that stores all module-related information.
    /// ```
    /// # use abstract_client::AbstractClientError;
    /// # let chain = cw_orch::prelude::MockBech32::new("mock");
    /// # let client = abstract_client::AbstractClient::builder(chain.clone()).build_mock().unwrap();
    /// use abstract_std::objects::{module_reference::ModuleReference, module::ModuleInfo};
    /// // For getting registry address
    /// use cw_orch::prelude::*;
    ///
    /// let registry = client.registry();
    /// let vc_module = registry.module(ModuleInfo::from_id_latest("abstract:registry")?)?;
    /// assert_eq!(vc_module.reference, ModuleReference::Native(registry.address()?));
    /// # Ok::<(), AbstractClientError>(())
    /// ```
    pub fn registry(&self) -> &Registry<Chain> {
        &self.abstr.registry
    }

    /// Abstract Name Service contract API
    ///
    /// The Abstract Name Service contract is a database contract that stores all asset-related information.
    /// ```
    /// # use abstract_client::AbstractClientError;
    /// # use abstract_testing::prelude::*;
    /// use abstract_client::{AbstractClient, ClientResolve};
    /// use cw_asset::AssetInfo;
    /// use abstract_app::objects::AssetEntry;
    /// // For getting registry address
    /// use cw_orch::prelude::*;
    ///
    /// let denom = "test_denom";
    /// let entry = "denom";
    /// # let chain = MockBech32::new("mock");
    /// # let client = AbstractClient::builder(chain.clone())
    /// #     .asset(entry, cw_asset::AssetInfoBase::Native(denom.to_owned()))
    /// #     .build_mock()?;
    ///
    /// let name_service = client.name_service();
    /// let asset_entry = AssetEntry::new(entry);
    /// let asset = asset_entry.resolve(name_service)?;
    /// assert_eq!(asset, AssetInfo::Native(denom.to_owned()));
    /// # Ok::<(), AbstractClientError>(())
    /// ```
    pub fn name_service(&self) -> &AnsHost<Chain> {
        &self.abstr.ans_host
    }

    /// Abstract Module Factory contract API
    pub fn module_factory(&self) -> &ModuleFactory<Chain> {
        &self.abstr.module_factory
    }

    /// Abstract Ibc Client contract API
    ///
    /// The Abstract Ibc Client contract allows users to create and use Interchain Abstract Accounts
    pub fn ibc_client(&self) -> &IbcClient<Chain> {
        &self.abstr.ibc.client
    }

    /// Service contract API
    pub fn service<M: RegisteredModule + From<Contract<Chain>>>(
        &self,
    ) -> AbstractClientResult<Service<Chain, M>> {
        Service::new(self.registry())
    }

    /// Return current block info see [`BlockInfo`].
    pub fn block_info(&self) -> AbstractClientResult<BlockInfo> {
        self.environment()
            .block_info()
            .map_err(|e| AbstractClientError::CwOrch(e.into()))
    }

    /// Publisher builder for creating new [`Publisher`](crate::Publisher) Abstract Account
    /// To publish any modules your account requires to have claimed a namespace.
    pub fn publisher_builder(&self, namespace: Namespace) -> PublisherBuilder<Chain> {
        PublisherBuilder::new(AccountBuilder::new(&self.abstr), namespace)
    }

    /// Builder for creating a new Abstract [`Account`].
    pub fn account_builder(&self) -> AccountBuilder<Chain> {
        AccountBuilder::new(&self.abstr)
    }

    /// Address of the sender
    pub fn sender(&self) -> Addr {
        self.environment().sender_addr()
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
        source: T,
    ) -> AbstractClientResult<Account<Chain>> {
        let source = source.into();
        let chain = self.abstr.registry.environment();

        match source {
            AccountSource::Namespace(namespace) => {
                // if namespace, check if we need to claim or not.
                // Check if namespace already claimed
                let account_from_namespace_result: Option<Account<Chain>> =
                    Account::maybe_from_namespace(&self.abstr, namespace.clone(), true)?;

                // Only return if the account can be retrieved without errors.
                if let Some(account_from_namespace) = account_from_namespace_result {
                    Ok(account_from_namespace)
                } else {
                    Err(AbstractClientError::NamespaceNotClaimed {
                        namespace: namespace.to_string(),
                    })
                }
            }
            AccountSource::AccountId(account_id) => {
                let abstract_account = AccountI::load_from(&self.abstr, account_id.clone())?;
                Ok(Account::new(abstract_account, true))
            }
            AccountSource::App(app) => {
                // Query app for account address and get AccountId from it.
                let app_config: abstract_std::app::AppConfigResponse = chain
                    .query(
                        &abstract_std::app::QueryMsg::<Empty>::Base(
                            abstract_std::app::BaseQueryMsg::BaseConfig {},
                        ),
                        &app,
                    )
                    .map_err(Into::into)?;

                let account_config: abstract_std::account::ConfigResponse = chain
                    .query(
                        &abstract_std::account::QueryMsg::Config {},
                        &app_config.account,
                    )
                    .map_err(Into::into)?;
                // This function verifies the account-id is valid and returns an error if not.
                let abstract_account = AccountI::load_from(&self.abstr, account_config.account_id)?;
                Ok(Account::new(abstract_account, true))
            }
        }
    }

    /// Retrieve denom balance for provided address
    pub fn query_balance(
        &self,
        address: &Addr,
        denom: impl Into<String>,
    ) -> AbstractClientResult<Uint128> {
        let coins = self
            .environment()
            .bank_querier()
            .balance(address, Some(denom.into()))
            .map_err(Into::into)?;
        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    /// Retrieve balances of all denoms for provided address
    pub fn query_balances(&self, address: &Addr) -> AbstractClientResult<Vec<Coin>> {
        self.environment()
            .bank_querier()
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

    /// Get random local account id sequence(unclaimed) in 2147483648..u32::MAX range
    pub fn random_account_id(&self) -> AbstractClientResult<u32> {
        let mut rng = rand::thread_rng();
        loop {
            let random_sequence = rng.gen_range(2147483648..u32::MAX);
            let potential_account_id = AccountId::local(random_sequence);
            if self.abstr.registry.account(potential_account_id).is_err() {
                return Ok(random_sequence);
            };
        }
    }

    /// Get address of instantiate2 module
    /// If used for upcoming account this supposed to be used in pair with [`AbstractClient::next_local_account_id`]
    pub fn module_instantiate2_address<M: RegisteredModule>(
        &self,
        account_id: &AccountId,
    ) -> AbstractClientResult<Addr> {
        self.module_instantiate2_address_raw(
            account_id,
            ModuleInfo::from_id(
                M::module_id(),
                ModuleVersion::Version(M::module_version().to_owned()),
            )?,
        )
    }

    /// Get address of instantiate2 module
    /// Raw version of [`AbstractClient::module_instantiate2_address`]
    /// If used for upcoming account this supposed to be used in pair with [`AbstractClient::next_local_account_id`]
    pub fn module_instantiate2_address_raw(
        &self,
        account_id: &AccountId,
        module_info: ModuleInfo,
    ) -> AbstractClientResult<Addr> {
        let salt = generate_instantiate_salt(account_id);
        let wasm_querier = self.environment().wasm_querier();
        let module = self.registry().module(module_info)?;
        let (code_id, creator) = match module.reference {
            // If Account - signer is creator
            ModuleReference::Account(id) => (id, self.environment().sender_addr()),
            // Else module factory is creator
            ModuleReference::App(id) | ModuleReference::Standalone(id) => {
                (id, self.abstr.module_factory.address()?)
            }
            _ => {
                return Err(AbstractClientError::Abstract(
                    abstract_std::AbstractError::Assert(
                        "module reference not account, app or standalone".to_owned(),
                    ),
                ))
            }
        };

        let addr = wasm_querier
            .instantiate2_addr(code_id, &creator, salt)
            .map_err(Into::into)?;
        Ok(Addr::unchecked(addr))
    }

    /// Retrieves the status of a specified module.
    ///
    /// This function checks the status of a module within the registry contract.
    /// and returns appropriate `Some(ModuleStatus)`. If the module is not deployed, it returns `None`.
    pub fn module_status(&self, module: ModuleInfo) -> AbstractClientResult<Option<ModuleStatus>> {
        self.registry().module_status(module).map_err(Into::into)
    }

    #[cfg(feature = "interchain")]
    /// Connect this abstract client to the remote abstract client
    /// If [`cw_orch_polytone::Polytone`] is deployed between 2 chains, it will NOT redeploy it (good for actual chains)
    /// If Polytone is not deployed, deploys it between the 2 chains (good for integration testing)
    pub fn connect_to(
        &self,
        remote_abstr: &AbstractClient<Chain>,
        ibc: &impl cw_orch_interchain::prelude::InterchainEnv<Chain>,
    ) -> AbstractClientResult<()>
    where
        Chain: cw_orch_interchain::prelude::IbcQueryHandler,
    {
        self.abstr.connect_to(&remote_abstr.abstr, ibc)?;

        Ok(())
    }
}

impl<Chain: CwEnv<Sender = Addr>> AbstractClient<Chain> {
    /// Admin of the abstract deployment
    pub fn mock_admin(chain: &Chain) -> <Chain as TxHandler>::Sender {
        Abstract::mock_admin(chain)
    }
}
