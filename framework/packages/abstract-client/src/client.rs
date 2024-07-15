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

use abstract_interface::{
    Abstract, AbstractAccount, AnsHost, IbcClient, ManagerQueryFns, RegisteredModule, VCQueryFns,
    VersionControl,
};
use abstract_std::objects::{
    module::{ModuleInfo, ModuleVersion},
    module_reference::ModuleReference,
    namespace::Namespace,
    salt::generate_instantiate_salt,
    AccountId,
};
use cosmwasm_std::{BlockInfo, Uint128};
use cw_orch::{environment::Environment as _, prelude::*};
use rand::Rng;

use crate::{
    account::{Account, AccountBuilder},
    source::AccountSource,
    AbstractClientError, Environment, PublisherBuilder,
};

/// Client to interact with Abstract accounts and modules
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
    /// # let client = AbstractClient::builder(chain.clone()).build().unwrap(); // Deploy mock abstract
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
    /// # let client = abstract_client::AbstractClient::builder(chain).build().unwrap();
    /// use abstract_std::objects::{module_reference::ModuleReference, module::ModuleInfo};
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

    /// Abstract Name Service contract API
    ///
    /// The Abstract Name Service contract is a database contract that stores all asset-related information.
    /// ```
    /// # use abstract_client::AbstractClientError;
    /// use abstract_client::{AbstractClient, ClientResolve};
    /// use cw_asset::AssetInfo;
    /// use abstract_app::objects::AssetEntry;
    /// // For getting version control address
    /// use cw_orch::prelude::*;
    ///
    /// let denom = "test_denom";
    /// let entry = "denom";
    /// # let client = AbstractClient::builder(MockBech32::new("mock"))
    /// #     .asset(entry, cw_asset::AssetInfoBase::Native(denom.to_owned()))
    /// #     .build()?;
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

    /// Abstract Ibc Client contract API
    ///
    /// The Abstract Ibc Client contract allows users to create and use Interchain Abstract Accounts
    pub fn ibc_client(&self) -> &IbcClient<Chain> {
        &self.abstr.ibc.client
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
        let chain = self.abstr.version_control.environment();

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
                let abstract_account: AbstractAccount<Chain> =
                    AbstractAccount::new(&self.abstr, account_id.clone());
                Ok(Account::new(abstract_account, true))
            }
            AccountSource::App(app) => {
                // Query app for manager address and get AccountId from it.
                let app_config: abstract_std::app::AppConfigResponse = chain
                    .query(
                        &abstract_std::app::QueryMsg::<Empty>::Base(
                            abstract_std::app::BaseQueryMsg::BaseConfig {},
                        ),
                        &app,
                    )
                    .map_err(Into::into)?;

                let manager_config: abstract_std::manager::ConfigResponse = chain
                    .query(
                        &abstract_std::manager::QueryMsg::Config {},
                        &app_config.manager_address,
                    )
                    .map_err(Into::into)?;
                // This function verifies the account-id is valid and returns an error if not.
                let abstract_account: AbstractAccount<Chain> =
                    AbstractAccount::new(&self.abstr, manager_config.account_id);
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

    // Retrieve the last account created by the client.
    /// Returns `None` if no account has been created yet.
    /// **Note**: This only returns accounts that were created with the Client. Any accounts created through the web-app will not be returned.
    pub fn get_last_account(&self) -> AbstractClientResult<Option<Account<Chain>>> {
        let addresses = self.environment().state().get_all_addresses()?;
        // Now search for all the keys that start with "abstract:manager-x" and return the one which has the highest x.
        let mut last_account: Option<(u32, Account<Chain>)> = None;
        for id in addresses.keys() {
            let Some(account_id) = is_local_manager(id.as_str())? else {
                continue;
            };

            // only take accounts that the current sender owns
            let account = AbstractAccount::new(&self.abstr, account_id.clone());
            if account.manager.top_level_owner()?.address != self.environment().sender_addr() {
                continue;
            }

            if let Some((last_account_id, _)) = last_account {
                if account_id.seq() > last_account_id {
                    last_account = Some((account_id.seq(), Account::new(account, true)));
                }
            } else {
                last_account = Some((account_id.seq(), Account::new(account, true)));
            }
        }
        Ok(last_account.map(|(_, account)| account))
    }

    /// Get random local account id sequence(unclaimed) in 2147483648..u32::MAX range
    pub fn random_account_id(&self) -> AbstractClientResult<u32> {
        let mut rng = rand::thread_rng();
        loop {
            let random_sequence = rng.gen_range(2147483648..u32::MAX);
            let potential_account_id = AccountId::local(random_sequence);
            if self
                .abstr
                .version_control
                .account_base(potential_account_id)
                .is_err()
            {
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
        let module = self.version_control().module(module_info)?;
        let (code_id, creator) = match module.reference {
            // If AccountBase - account factory is creator
            ModuleReference::AccountBase(id) => (id, self.abstr.account_factory.addr_str()?),
            // Else module factory is creator
            ModuleReference::App(id) | ModuleReference::Standalone(id) => {
                (id, self.abstr.module_factory.addr_str()?)
            }
            _ => {
                return Err(AbstractClientError::Abstract(
                    abstract_std::AbstractError::Assert(
                        "module reference not account base, app or standalone".to_owned(),
                    ),
                ))
            }
        };

        let addr = wasm_querier
            .instantiate2_addr(code_id, creator, salt)
            .map_err(Into::into)?;
        Ok(Addr::unchecked(addr))
    }

    #[cfg(feature = "interchain")]
    /// Connect this abstract client to the remote abstract client
    /// If [`cw_orch_polytone::Polytone`] is deployed between 2 chains, it will NOT redeploy it (good for actual chains)
    /// If Polytone is not deployed, deploys it between the 2 chains (good for integration testing)
    pub fn connect_to(
        &self,
        remote_abstr: &AbstractClient<Chain>,
        ibc: &impl cw_orch_interchain::InterchainEnv<Chain>,
    ) -> AbstractClientResult<()>
    where
        Chain: cw_orch_interchain::IbcQueryHandler,
    {
        self.abstr.connect_to(&remote_abstr.abstr, ibc)?;

        Ok(())
    }
}

pub(crate) fn is_local_manager(id: &str) -> AbstractClientResult<Option<AccountId>> {
    if !id.starts_with(abstract_std::MANAGER) {
        return Ok(None);
    }

    let (_, account_id_str) = id.split_once('-').unwrap();
    let account_id = AccountId::try_from(account_id_str)?;

    // Only take local accounts into account.
    if account_id.is_remote() {
        return Ok(None);
    }

    Ok(Some(account_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_account() {
        let result = is_local_manager("abstract:manager-local-9");
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn remote_account() {
        let result = is_local_manager("abstract:manager-eth>btc-9");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn not_manager() {
        let result = is_local_manager("abstract:proxy-local-9");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn last_owned_abstract_account() {
        let chain = MockBech32::new("mock");
        let sender = chain.sender_addr();
        Abstract::deploy_on(chain.clone(), sender.to_string()).unwrap();

        let client = AbstractClient::new(chain.clone()).unwrap();
        let _acc = client.account_builder().build().unwrap();
        let acc_2 = client.account_builder().build().unwrap();

        let other_owner = chain.addr_make("other_owner");
        // create account with sender as sender but other owner
        client
            .account_builder()
            .ownership(
                abstract_std::objects::gov_type::GovernanceDetails::Monarchy {
                    monarch: other_owner.to_string(),
                },
            )
            .build()
            .unwrap();

        let last_account = client.get_last_account().unwrap().unwrap();

        assert_eq!(acc_2.id().unwrap(), last_account.id().unwrap());
    }
}
