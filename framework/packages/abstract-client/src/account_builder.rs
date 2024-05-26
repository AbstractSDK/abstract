use std::{fmt::{Debug, Display}, marker::PhantomData};

use abstract_interface::{
    Abstract, AbstractAccount, AbstractInterfaceError, AccountDetails, DependencyCreation,
    IbcClient, InstallConfig, MFactoryQueryFns, ManagerExecFns, ManagerQueryFns, RegisteredModule,
    VCQueryFns,
};
use abstract_std::{
    ibc_client::QueryMsgFns,
    manager::{
        state::AccountInfo, InfoResponse, ManagerModuleInfo, ModuleAddressesResponse,
        ModuleInfosResponse, ModuleInstallConfig,
    },
    objects::{
        chain_name::ChainName,
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion},
        namespace::Namespace,
        nested_admin::MAX_ADMIN_RECURSION,
        validation::verifiers,
        AccountId, AssetEntry,
    },
    version_control::NamespaceResponse,
    IBC_CLIENT, PROXY,
};
use cosmwasm_std::{to_json_binary, Attribute, Coins, CosmosMsg, Uint128};
use cw_orch::{contract::Contract, environment::MutCwEnv, prelude::*};

use crate::{
    client::AbstractClientResult,
    infrastructure::{Environment, Infrastructure},
    remote_account::RemoteAccount,
    AbstractClientError, Account, Application,
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
pub struct AccountBuilder<'a, Chain: CwEnv, AccType = NewAcc> {
    pub(crate) abstr: &'a Abstract<Chain>,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    namespace: Option<Namespace>,
    base_asset: Option<AssetEntry>,
    // TODO: Decide if we want to abstract this as well.
    ownership: Option<GovernanceDetails<String>>,
    owner_account: Option<OwnerAccount<'a, Chain>>,
    install_modules: Vec<ModuleInstallConfig>,
    funds: AccountCreationFunds,
    fetch_if_namespace_claimed: bool,
    install_on_sub_account: bool,
    expected_local_account_id: Option<u32>,
    // Signal type for builder implementation
    account_type: PhantomData<AccType>,
}

/// Creation funds
enum AccountCreationFunds {
    #[allow(clippy::type_complexity)]
    Auto(Box<dyn Fn(&[Coin]) -> bool>),
    Coins(Coins),
}

/// Owner of created account
enum OwnerAccount<'a, Chain: CwEnv> {
    CurrentChain(&'a Account<Chain>),
    RemoteChain(&'a Account<Chain>),
}

pub (crate) struct RemoteAcc;
pub (crate) struct SubAcc;
pub (crate) struct NewAcc;

impl<'a, Chain: CwEnv, AccType> AccountBuilder<'a, Chain, AccType> {
    pub(crate) fn new(abstr: &'a Abstract<Chain>) -> Self {
        Self {
            abstr,
            name: None,
            description: None,
            link: None,
            namespace: None,
            base_asset: None,
            ownership: None,
            owner_account: None,
            account_type: PhantomData::default(),
            install_modules: vec![],
            funds: AccountCreationFunds::Coins(Coins::default()),
            fetch_if_namespace_claimed: true,
            install_on_sub_account: true,
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

    /// Base Asset for the account
    pub fn base_asset(&mut self, base_asset: AssetEntry) -> &mut Self {
        self.base_asset = Some(base_asset);
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
    pub fn sub_account(&mut self, owner_account: &'a Account<Chain>) -> AccountBuilder<'_, Chain, SubAcc> {
        let builder: AccountBuilder<'a, Chain, SubAcc> = Self {
            abstr: self.abstr,
            name: self.name,
            description: self.description,
            link: self.,
            namespace: self.,
            base_asset: self.,
            ownership: self.,
            owner_account: Some(OwnerAccount::CurrentChain(owner_account)),
            account_type: PhantomData::default(),
            install_modules: vec![],
            funds: AccountCreationFunds::Coins(Coins::default()),
            fetch_if_namespace_claimed: true,
            install_on_sub_account: false,
            expected_local_account_id: None,
        }
    }

    /// Create remote-account instead
    /// And set install_on_sub_account to false to prevent installing on sub account of the remote account
    pub fn remote_account(&mut self, owner_account: &'a Account<Chain>) -> &mut Self {
        self.owner_account = Some(OwnerAccount::RemoteChain(owner_account));
        self.install_on_sub_account = false;
        self.account_type = Some(RemoteAcc);
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
    /// The tx will error if this does not match the account-id at runtime. Useful for instantiate2 address prediction.
    pub fn expected_account_id(&mut self, local_account_id: u32) -> &mut Self {
        self.expected_local_account_id = Some(local_account_id);
        self
    }
}

/// Builds the [`Account`].
fn _build<Chain: CwEnv, AccType>(
    builder: &AccountBuilder<'_, Chain, AccType>,
) -> AbstractClientResult<Account<Chain>> {
    if builder.fetch_if_namespace_claimed {
        // Check if namespace already claimed
        if let Some(ref namespace) = builder.namespace {
            let account_from_namespace_result: Option<Account<Chain>> =
                Account::maybe_from_namespace(
                    builder.abstr,
                    namespace.clone(),
                    builder.install_on_sub_account,
                )?;

            // Only return if the account can be retrieved without errors.
            if let Some(account_from_namespace) = account_from_namespace_result {
                return Ok(account_from_namespace);
            }
        }
    }

    let chain = builder.abstr.version_control.get_chain();
    let sender = chain.sender().to_string();
    let name = builder
        .name
        .clone()
        .unwrap_or_else(|| String::from("Default Abstract Account"));
    let ownership = builder
        .ownership
        .clone()
        .unwrap_or(GovernanceDetails::Monarchy { monarch: sender });

    // Validate everything before sending tx
    verifiers::validate_name(&name)?;
    verifiers::validate_description(builder.description.as_deref())?;
    verifiers::validate_link(builder.link.as_deref())?;

    let install_modules = builder.install_modules.clone();
    let funds = match &builder.funds {
        AccountCreationFunds::Auto(auto_funds_assert) => {
            let modules = install_modules.iter().map(|m| m.module.clone()).collect();
            // Simulate module install to find out required funds
            let simulate_response = builder
                .abstr
                .module_factory
                .simulate_install_modules(modules)?;

            let mut funds = Coins::try_from(simulate_response.total_required_funds).unwrap();

            // Add namespace fee if any
            if builder.namespace.is_some() {
                let vc_config = builder.abstr.version_control.config()?;

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

    let account_details = AccountDetails {
        name,
        description: builder.description.clone(),
        link: builder.link.clone(),
        namespace: builder.namespace.as_ref().map(ToString::to_string),
        base_asset: builder.base_asset.clone(),
        install_modules,
        account_id: builder.expected_local_account_id,
    };
    let abstract_account = match builder.owner_account {
        None => builder.abstr.account_factory.create_new_account(
            account_details,
            ownership,
            Some(&funds),
        )?,
        Some(OwnerAccount::CurrentChain(owner_account)) => owner_account
            .abstr_account
            .create_sub_account(account_details, Some(&funds))?,
        Some(OwnerAccount::RemoteChain(_)) => {
            return Err(AbstractClientError::RemoteAccountOnBuild {})
        }
    };
    Ok(Account::new(
        abstract_account,
        builder.install_on_sub_account,
    ))
}

/// Builds the [`RemoteAccount`].
/// Before using it you are supposed to wait Response.
/// For example: https://orchestrator.abstract.money/interchain/integrations/daemon.html?#analysis-usage
fn _build_remote<Chain: CwEnv>(builder: &AccountBuilder<'_, Chain, RemoteAcc>) -> AbstractClientResult<(RemoteAccount<Chain>, Chain::Response)> {
    let chain = builder.abstr.version_control.get_chain();
    let name = builder
        .name
        .clone()
        .unwrap_or_else(|| String::from("Default Abstract Account"));

    // Validate everything before sending tx
    verifiers::validate_name(&name)?;
    verifiers::validate_description(builder.description.as_deref())?;
    verifiers::validate_link(builder.link.as_deref())?;

    let Some(OwnerAccount::RemoteChain(owner_account)) = builder.owner_account else {
        return Err(AbstractClientError::LocalAccountOnBuildRemote {});
    };
    let env = chain.env_info();

    let mut install_modules = builder.install_modules.clone();
    // We add the IBC Client by default in the modules installed on the remote account
    if !install_modules.iter().any(|m| m.module.id() == IBC_CLIENT) {
        install_modules.push(ModuleInstallConfig::new(
            ModuleInfo::from_id_latest(IBC_CLIENT)?,
            None,
        ));
    }

    let account_details = AccountDetails {
        name,
        description: builder.description.clone(),
        link: builder.link.clone(),
        namespace: builder.namespace.as_ref().map(ToString::to_string),
        base_asset: builder.base_asset.clone(),
        install_modules,
        account_id: builder.expected_local_account_id,
    };
    let host_chain = ChainName::from_string(env.chain_name)?;

    let response = owner_account
        .abstr_account
        .create_remote_account(account_details, host_chain)?;

    let remote_account_id = {
        let mut id = owner_account.id()?;
        let chain_name = ChainName::from_string(
            owner_account
                .abstr_account
                .manager
                .get_chain()
                .env_info()
                .chain_name,
        )?;
        id.push_chain(chain_name);
        id
    };

    Ok((
        RemoteAccount::new(
            owner_account.abstr_account.clone(),
            remote_account_id,
            chain.clone(),
        ),
        response,
    ))
}

impl<Chain: CwEnv> AccountBuilder<'_, Chain, NewAcc> {
    /// Builds the [`Account`].
    pub fn build(&self) -> AbstractClientResult<Account<Chain>> {
        _build(self)
    }
}

impl<Chain: CwEnv> AccountBuilder<'_, Chain, RemoteAcc> {
    /// Builds the [`Account`].
    pub fn build(&self) -> AbstractClientResult<(RemoteAccount<Chain>, Chain::Response)> {
        _build_remote(self)
    }
}

impl<Chain: CwEnv> AccountBuilder<'_, Chain, SubAcc> {
    /// Builds the [`Account`].
    pub fn build(&self) -> AbstractClientResult<Account<Chain>> {
        _build(self)
    }
}
