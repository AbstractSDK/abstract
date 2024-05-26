//! # Represents Abstract Account
//!
//! [`Account`] allows you to interact with your or another user Abstract Account
//!
//! Example of creating an account
//! ```
//! # use abstract_client::AbstractClientError;
//! use abstract_client::{AbstractClient, Account};
//! use cw_orch::prelude::*;
//!
//! # let chain = MockBech32::new("mock");
//! # let client: AbstractClient<MockBech32> = AbstractClient::builder(chain).build()?;
//!
//! let alice_account: Account<MockBech32> = client
//!     .account_builder()
//!     .name("Alice")
//!     .build()?;
//!
//! assert_eq!(alice_account.owner()?, client.sender());
//! # Ok::<(), AbstractClientError>(())
//! ```
use std::fmt::{Debug, Display};

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
    AbstractClientError, Application,
};

/// Represents an existing Abstract account.
///
/// Get this struct from [`AbstractClient::account_from_namespace`](crate::AbstractClient)
/// or create a new account with the [`AccountBuilder`](crate::AbstractClient::account_builder).
#[derive(Clone)]
pub struct Account<Chain: CwEnv> {
    pub(crate) abstr_account: AbstractAccount<Chain>,
    install_on_sub_account: bool,
}

impl<Chain: CwEnv> AsRef<AbstractAccount<Chain>> for Account<Chain> {
    fn as_ref(&self) -> &AbstractAccount<Chain> {
        &self.abstr_account
    }
}

struct ParsedAccountCreationResponse {
    sub_account_id: u32,
    module_address: String,
}

impl<Chain: CwEnv> Account<Chain> {
    pub(crate) fn new(
        abstract_account: AbstractAccount<Chain>,
        install_on_sub_account: bool,
    ) -> Self {
        Self {
            abstr_account: abstract_account,
            install_on_sub_account,
        }
    }

    pub(crate) fn maybe_from_namespace(
        abstr: &Abstract<Chain>,
        namespace: Namespace,
        install_on_sub_account: bool,
    ) -> AbstractClientResult<Option<Self>> {
        let namespace_response: NamespaceResponse = abstr.version_control.namespace(namespace)?;

        let NamespaceResponse::Claimed(info) = namespace_response else {
            return Ok(None);
        };

        let abstract_account: AbstractAccount<Chain> = AbstractAccount::new(abstr, info.account_id);

        Ok(Some(Self::new(abstract_account, install_on_sub_account)))
    }

    /// Get the [`AccountId`] of the Account
    pub fn id(&self) -> AbstractClientResult<AccountId> {
        self.abstr_account.id().map_err(Into::into)
    }

    /// Wether this account installs his applications on a sub account
    pub fn install_on_sub_account(&self) -> bool {
        self.install_on_sub_account
    }

    // TODO:
    // pub fn deposit(
    //     &self,
    //     assets: Vec<AssetInfo>,
    // ) -> AbstractClientResult<<Chain as TxHandler>::Response> {
    // We need to try to batch it so if one of the deposits fail - we just fail tx
    // }

    /// Query account balance of a given denom
    // TODO: Asset balance?
    pub fn query_balance(&self, denom: impl Into<String>) -> AbstractClientResult<Uint128> {
        let coins = self
            .environment()
            .bank_querier()
            .balance(self.proxy()?, Some(denom.into()))
            .map_err(Into::into)?;

        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    /// Query account balances of all denoms
    pub fn query_balances(&self) -> AbstractClientResult<Vec<Coin>> {
        self.environment()
            .bank_querier()
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
    /// if `install_on_sub_account` is `true`, the application will be installed on new a sub-account. (default)
    pub fn install_app<M: InstallConfig + From<Contract<Chain>>>(
        &self,
        configuration: &M::InitMsg,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let modules = vec![M::install_config(configuration)?];

        match self.install_on_sub_account {
            true => self.install_module_sub_internal(modules, funds),
            false => self.install_module_current_internal(modules, funds),
        }
    }

    /// Install an adapter on current account.
    pub fn install_adapter<M: InstallConfig<InitMsg = Empty> + From<Contract<Chain>>>(
        &self,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let modules = vec![M::install_config(&Empty {})?];

        match self.install_on_sub_account {
            true => self.install_module_sub_internal(modules, funds),
            false => self.install_module_current_internal(modules, funds),
        }
    }

    /// Creates a new sub-account on the current account and
    /// installs an App module and its dependencies with the provided dependencies config. \
    ///
    /// The returned [`Application`] is a wrapper around the sub-account and simplifies interaction with the App module.
    pub fn install_app_with_dependencies<
        M: DependencyCreation + InstallConfig + From<Contract<Chain>>,
    >(
        &self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let mut install_configs: Vec<ModuleInstallConfig> =
            M::dependency_install_configs(dependencies_config)?;
        install_configs.push(M::install_config(module_configuration)?);

        match self.install_on_sub_account {
            true => self.install_module_sub_internal(install_configs, funds),
            false => self.install_module_current_internal(install_configs, funds),
        }
    }

    /// Upgrades the account to the latest version
    ///
    /// Migrates manager and proxy contracts to their respective new versions.
    pub fn upgrade(&self, version: ModuleVersion) -> AbstractClientResult<()> {
        self.abstr_account.manager.upgrade(vec![
            (
                ModuleInfo::from_id(abstract_std::registry::MANAGER, version.clone())?,
                Some(
                    to_json_binary(&abstract_std::manager::MigrateMsg {})
                        .map_err(Into::<CwOrchError>::into)?,
                ),
            ),
            (
                ModuleInfo::from_id(abstract_std::registry::PROXY, version)?,
                Some(
                    to_json_binary(&abstract_std::proxy::MigrateMsg {})
                        .map_err(Into::<CwOrchError>::into)?,
                ),
            ),
        ])?;
        Ok(())
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
                            &abstract_std::manager::QueryMsg::Info {},
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
                &abstract_std::manager::ExecuteMsg::ExecOnModule {
                    module_id: PROXY.to_owned(),
                    exec_msg: to_json_binary(&abstract_std::proxy::ExecuteMsg::ModuleAction {
                        msgs,
                    })
                    .map_err(AbstractInterfaceError::from)?,
                },
                Some(funds),
            )
            .map_err(Into::into)
    }

    /// Set IBC status on an Account.
    pub fn set_ibc_status(&self, enabled: bool) -> AbstractClientResult<()> {
        self.abstr_account.manager.update_settings(Some(enabled))?;

        Ok(())
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

    /// Get Sub Accounts of this account
    pub fn sub_accounts(&self) -> AbstractClientResult<Vec<Account<Chain>>> {
        let mut sub_accounts = vec![];
        let mut start_after = None;
        let abstr_deployment = Abstract::load_from(self.environment())?;
        loop {
            let sub_account_ids = self
                .abstr_account
                .manager
                .sub_account_ids(None, start_after)?
                .sub_accounts;
            start_after = sub_account_ids.last().cloned();

            if sub_account_ids.is_empty() {
                break;
            }
            sub_accounts.extend(sub_account_ids.into_iter().map(|id| {
                Account::new(
                    AbstractAccount::new(&abstr_deployment, AccountId::local(id)),
                    false,
                )
            }));
        }
        Ok(sub_accounts)
    }

    /// Get remote Account of this account
    pub fn remote_account(
        &self,
        remote_chain: Chain,
    ) -> AbstractClientResult<RemoteAccount<Chain>> {
        // Make sure ibc client installed on account
        let ibc_client = self.application::<IbcClient<Chain>>()?;
        let remote_chain_name = ChainName::from_string(remote_chain.env_info().chain_name)?;
        let account_id = self.id()?;

        // Check it exists first
        let remote_account_response =
            ibc_client.remote_account(account_id.clone(), remote_chain_name.clone())?;
        if remote_account_response.remote_proxy_addr.is_none() {
            return Err(AbstractClientError::RemoteAccountNotFound {
                account_id,
                chain: remote_chain_name,
                ibc_client_addr: ibc_client.address()?,
            });
        }

        // Now structure remote account
        let owner_account = self.abstr_account.clone();

        let remote_account_id = {
            let mut id = owner_account.id()?;
            let chain_name =
                ChainName::from_string(owner_account.manager.get_chain().env_info().chain_name)?;
            id.push_chain(chain_name);
            id
        };

        Ok(RemoteAccount::new(
            owner_account,
            remote_account_id,
            remote_chain,
        ))
    }

    /// Address of the proxy
    pub fn proxy(&self) -> AbstractClientResult<Addr> {
        self.abstr_account.proxy.address().map_err(Into::into)
    }

    /// Address of the manager
    pub fn manager(&self) -> AbstractClientResult<Addr> {
        self.abstr_account.manager.address().map_err(Into::into)
    }

    /// Retrieve installed application on account
    /// This can't retrieve sub-account installed applications.
    pub fn application<M: RegisteredModule + From<Contract<Chain>>>(
        &self,
    ) -> AbstractClientResult<Application<Chain, M>> {
        let module = self.module()?;
        let account = self.clone();

        Application::new(account, module)
    }

    /// Install module on current account
    fn install_module_current_internal<M: RegisteredModule + From<Contract<Chain>>>(
        &self,
        modules: Vec<ModuleInstallConfig>,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let install_module_response = self
            .abstr_account
            .manager
            .install_modules(modules, Some(funds))?;

        let module_addr = Self::parse_modules_installing_response(install_module_response);
        let contract = Contract::new(
            M::installed_module_contract_id(&self.id()?),
            self.environment(),
        );
        contract.set_address(&module_addr);

        let adapter: M = contract.into();

        Application::new(
            Account::new(self.abstr_account.clone(), self.install_on_sub_account),
            adapter,
        )
    }

    /// Installs module on sub account
    fn install_module_sub_internal<M: RegisteredModule + From<Contract<Chain>>>(
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
            None,
            funds,
        )?;

        let parsed_account_creation_response =
            Self::parse_account_creation_response(sub_account_response);

        let sub_account: AbstractAccount<Chain> = AbstractAccount::new(
            &self.infrastructure()?,
            AccountId::local(parsed_account_creation_response.sub_account_id),
        );

        let contract = Contract::new(
            M::installed_module_contract_id(&sub_account.id()?),
            self.environment(),
        );
        contract.set_address(&Addr::unchecked(
            parsed_account_creation_response.module_address,
        ));

        let app: M = contract.into();

        Application::new(Account::new(sub_account, false), app)
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

    fn parse_modules_installing_response(response: <Chain as TxHandler>::Response) -> Addr {
        let wasm_abstract_attributes: Vec<Attribute> = response
            .events()
            .into_iter()
            .filter(|e| e.ty == "wasm-abstract")
            .flat_map(|e| e.attributes)
            .collect();

        let module_addresses: String = wasm_abstract_attributes
            .iter()
            .find(|a| a.key == "new_modules")
            .map(|a| a.value.parse().unwrap())
            .unwrap();

        // When there are multiple modules registered the addresses are returned in a common
        // separated list. We want the last one as that is the "top-level" module while the rest
        // are dependencies, since in the sub-account creation call, we pass in the top-level
        // module last.
        let module_address = module_addresses.split(',').last().unwrap();

        // We install only one module
        Addr::unchecked(module_address)
    }

    pub(crate) fn module<T: RegisteredModule + From<Contract<Chain>>>(
        &self,
    ) -> AbstractClientResult<T> {
        let module_id = T::module_id();
        let account_module_id = T::installed_module_contract_id(&self.id()?);
        let maybe_module_addr = self.module_addresses(vec![module_id.to_string()])?.modules;

        if !maybe_module_addr.is_empty() {
            let contract = Contract::new(account_module_id, self.environment());
            contract.set_address(&maybe_module_addr[0].1);
            let module: T = contract.into();
            Ok(module)
        } else {
            Err(AbstractClientError::ModuleNotInstalled {})
        }
    }

    /// Claim a namespace for an existing account
    pub fn claim_namespace(
        &self,
        namespace: impl Into<String>,
    ) -> Result<Chain::Response, AbstractInterfaceError> {
        self.abstr_account.claim_namespace(namespace)
    }
}

impl<Chain: MutCwEnv> Account<Chain> {
    /// Set balance for the Proxy
    pub fn set_balance(&self, amount: &[Coin]) -> AbstractClientResult<()> {
        self.environment()
            .set_balance(&self.proxy()?, amount.to_vec())
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Add balance to the Proxy
    pub fn add_balance(&self, amount: &[Coin]) -> AbstractClientResult<()> {
        self.environment()
            .add_balance(&self.proxy()?, amount.to_vec())
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

impl<Chain: CwEnv> Display for Account<Chain> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abstr_account)
    }
}

impl<Chain: CwEnv> Debug for Account<Chain> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
        // TODO:
        // write!(f, "{:?}", self.abstr_account)
    }
}

#[cfg(test)]
pub mod test {
    use abstract_interface::{Abstract, VCQueryFns};
    use abstract_std::objects::namespace::Namespace;
    use cw_orch::{contract::Deploy, mock::MockBech32};

    use crate::AbstractClient;

    #[test]
    fn namespace_after_creation() -> cw_orch::anyhow::Result<()> {
        let mock = MockBech32::new("mock");
        let abstr = AbstractClient::builder(mock.clone()).build()?;

        let my_namespace = "my-namespace";
        let new_account = abstr.account_builder().build()?;
        new_account.claim_namespace(my_namespace)?;

        // Verify the namespace exists
        let abstr = Abstract::load_from(mock.clone())?;
        let namespace_response = abstr
            .version_control
            .namespace(Namespace::new(my_namespace)?)?;

        match namespace_response {
            abstract_std::version_control::NamespaceResponse::Claimed(c) => {
                assert_eq!(c.account_id, new_account.id()?)
            }
            abstract_std::version_control::NamespaceResponse::Unclaimed {} => {
                panic!("Expected claimed namespace")
            }
        }

        Ok(())
    }
}
