//! # Represents Remote Abstract Account
//!
//! [`RemoteAccount`] allows you to interact with your or another user remote Abstract Account
//!

use abstract_interface::{
    Abstract, AccountDetails, AccountI, AccountQueryFns as _, DependencyCreation, IbcClient,
    InstallConfig, RegisteredModule, RegistryQueryFns as _,
};
use abstract_std::{
    account::{
        self, state::AccountInfo, AccountModuleInfo, InfoResponse, ModuleAddressesResponse,
        ModuleInfosResponse, ModuleInstallConfig,
    },
    ibc_client::{self, QueryMsgFns as _},
    ibc_host,
    objects::{
        module::{ModuleId, ModuleInfo, ModuleVersion},
        namespace::Namespace,
        ownership, AccountId, TruncatedChainId,
    },
    IBC_CLIENT,
};
use cosmwasm_std::{to_json_binary, CosmosMsg, Uint128};
use cw_orch::{
    contract::Contract,
    environment::{Environment as _, MutCwEnv},
    prelude::*,
};
use cw_orch_interchain::{IbcQueryHandler, InterchainEnv};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    client::AbstractClientResult, AbstractClient, AbstractClientError, Account, Environment,
    IbcTxAnalysisV2, RemoteApplication,
};

/// A builder for creating [`RemoteAccounts`](RemoteAccount).
/// Get the builder from the [`AbstractClient::Account`](crate::Account)
/// and create the account with the `build` method.
pub struct RemoteAccountBuilder<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> {
    pub(crate) ibc_env: IBC,
    pub(crate) host_chain: Chain,
    namespace: Option<Namespace>,
    owner_account: Account<Chain>,
    install_modules: Vec<ModuleInstallConfig>,
    // TODO: how we want to manage funds ibc-wise?
    // funds: AccountCreationFunds,
}

impl<Chain: IbcQueryHandler> Account<Chain> {
    /// Builder for creating a new Abstract [`RemoteAccount`].
    pub fn remote_account_builder<IBC: InterchainEnv<Chain>>(
        &self,
        interchain_env: IBC,
        host_abstract: &AbstractClient<Chain>,
    ) -> RemoteAccountBuilder<Chain, IBC> {
        RemoteAccountBuilder::new(self.clone(), interchain_env, host_abstract.environment())
    }

    /// Get [`RemoteAccount`] of this account
    pub fn remote_account<IBC: InterchainEnv<Chain>>(
        &self,
        interchain_env: IBC,
        host_chain: Chain,
    ) -> AbstractClientResult<RemoteAccount<Chain, IBC>> {
        // Make sure ibc client installed on account
        let ibc_client = self.application::<IbcClient<Chain>>()?;
        let host_chain_name = TruncatedChainId::from_chain_id(&host_chain.chain_id());
        let account_id = self.id()?;

        // Check it exists first
        let remote_account_response =
            ibc_client.remote_account(account_id.clone(), host_chain_name.clone())?;
        if remote_account_response.remote_account_addr.is_none() {
            return Err(AbstractClientError::RemoteAccountNotFound {
                account_id,
                chain: host_chain_name,
                ibc_client_addr: ibc_client.address()?,
            });
        }

        // Now structure remote account
        let owner_account = self.abstr_account.clone();

        let remote_account_id = {
            let mut id = owner_account.id()?;
            let chain_name =
                TruncatedChainId::from_chain_id(&owner_account.environment().chain_id());
            id.push_chain(chain_name);
            id
        };

        Ok(RemoteAccount::new(
            owner_account,
            remote_account_id,
            host_chain,
            interchain_env,
        ))
    }
}

impl<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> RemoteAccountBuilder<Chain, IBC> {
    pub(crate) fn new(owner_account: Account<Chain>, ibc_env: IBC, host_chain: Chain) -> Self {
        Self {
            ibc_env,
            host_chain,
            namespace: None,
            owner_account,
            install_modules: vec![],
        }
    }

    /// Unique namespace for the account
    /// Setting this will claim the namespace for the account on construction.
    pub fn namespace(mut self, namespace: Namespace) -> Self {
        self.namespace = Some(namespace);
        self
    }

    /// Install an adapter on current account.
    pub fn install_adapter<M: InstallConfig<InitMsg = Empty>>(
        mut self,
    ) -> AbstractClientResult<Self> {
        self.install_modules.push(M::install_config(&Empty {})?);
        Ok(self)
    }

    /// Install an application on current account.
    pub fn install_app<M: InstallConfig>(
        mut self,
        configuration: &M::InitMsg,
    ) -> AbstractClientResult<Self> {
        self.install_modules.push(M::install_config(configuration)?);
        Ok(self)
    }

    /// Install an application with dependencies on current account.
    pub fn install_app_with_dependencies<M: DependencyCreation + InstallConfig>(
        mut self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
    ) -> AbstractClientResult<Self> {
        let deps_install_config = M::dependency_install_configs(dependencies_config)?;
        self.install_modules.extend(deps_install_config);
        self.install_modules
            .push(M::install_config(module_configuration)?);
        Ok(self)
    }

    /// Builds the [`RemoteAccount`].
    /// Before using it you are supposed to wait Response.
    /// For example: https://orchestrator.abstract.money/interchain/integrations/daemon.html?#analysis-usage
    pub fn build(self) -> AbstractClientResult<RemoteAccount<Chain, IBC>> {
        let host_chain = self.host_chain;
        let host_env_info = host_chain.env_info();

        let owner_account = self.owner_account;
        let env_info = owner_account.environment().env_info();

        let mut install_modules = self.install_modules.clone();
        // We add the IBC Client by default in the modules installed on the remote account
        if !install_modules.iter().any(|m| m.module.id() == IBC_CLIENT) {
            install_modules.push(ModuleInstallConfig::new(
                ModuleInfo::from_id_latest(IBC_CLIENT)?,
                None,
            ));
        }

        let account_details = AccountDetails {
            namespace: self.namespace.as_ref().map(ToString::to_string),
            install_modules,
            ..Default::default()
        };
        let host_chain_id = TruncatedChainId::from_chain_id(&host_env_info.chain_id);

        let response = owner_account
            .abstr_account
            .create_remote_account(account_details, host_chain_id)?;
        self.ibc_env
            .await_and_check_packets(&env_info.chain_id, response)?;

        let remote_account_id = {
            let mut id = owner_account.id()?;
            let chain_name = TruncatedChainId::from_chain_id(
                &owner_account.abstr_account.environment().chain_id(),
            );
            id.push_chain(chain_name);
            id
        };

        Ok(RemoteAccount::new(
            owner_account.abstr_account,
            remote_account_id,
            host_chain,
            self.ibc_env,
        ))
    }
}

/// Represents an existing remote Abstract account.
///
/// Get this struct from [`Account::remote_account`](crate::Account::remote_account)
/// or create a new account with the [`AccountBuilder`](crate::AbstractClient::account_builder).
///
/// Any execution done on remote account done through source chain
#[derive(Clone)]
pub struct RemoteAccount<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> {
    pub(crate) abstr_owner_account: AccountI<Chain>,
    remote_account_id: AccountId,
    host_chain: Chain,
    ibc_env: IBC,
}

impl<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> RemoteAccount<Chain, IBC> {
    pub(crate) fn new(
        abstr_owner_account: AccountI<Chain>,
        remote_account_id: AccountId,
        host_chain: Chain,
        ibc_env: IBC,
    ) -> Self {
        Self {
            abstr_owner_account,
            remote_account_id,
            host_chain,
            ibc_env,
        }
    }

    /// Get the [`AccountId`] of the Account
    pub fn id(&self) -> AccountId {
        self.remote_account_id.clone()
    }

    /// Truncated chain id of the host chain
    pub fn host_chain_id(&self) -> TruncatedChainId {
        TruncatedChainId::from_chain_id(&self.host_chain().env_info().chain_id)
    }

    fn host_chain(&self) -> Chain {
        self.host_chain.clone()
    }

    fn origin_chain(&self) -> Chain {
        self.abstr_owner_account.environment().clone()
    }

    /// Address of the account
    pub fn address(&self) -> AbstractClientResult<Addr> {
        let base_response = self
            .host_abstract()?
            .registry
            .account(self.remote_account_id.clone())?;

        Ok(base_response.account.addr().clone())
    }

    /// Query account balance of a given denom
    pub fn query_balance(&self, denom: impl Into<String>) -> AbstractClientResult<Uint128> {
        let coins = self
            .host_chain()
            .bank_querier()
            .balance(&self.address()?, Some(denom.into()))
            .map_err(Into::into)?;

        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    /// Query account balances of all denoms
    pub fn query_balances(&self) -> AbstractClientResult<Vec<Coin>> {
        self.host_chain()
            .bank_querier()
            .balance(&self.address()?, None)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Query account info
    pub fn info(&self) -> AbstractClientResult<AccountInfo> {
        let info_response: InfoResponse = self
            .host_chain()
            .query(&account::QueryMsg::Info {}, &self.address()?)
            .map_err(Into::into)?;
        Ok(info_response.info)
    }

    /// Install an application on account.
    pub fn install_app<
        M: RegisteredModule
            + From<Contract<Chain>>
            + ExecutableContract
            + QueryableContract
            + ContractInstance<Chain>
            + InstallConfig,
    >(
        &self,
        configuration: &M::InitMsg,
    ) -> AbstractClientResult<RemoteApplication<Chain, IBC, M>> {
        let modules = vec![M::install_config(configuration)?];

        self.install_module_host_internal(modules)
    }

    /// Install an adapter on account.
    pub fn install_adapter<
        M: RegisteredModule
            + From<Contract<Chain>>
            + ExecutableContract
            + QueryableContract
            + ContractInstance<Chain>
            + InstallConfig<InitMsg = Empty>,
    >(
        &self,
    ) -> AbstractClientResult<RemoteApplication<Chain, IBC, M>> {
        let modules = vec![M::install_config(&cosmwasm_std::Empty {})?];

        self.install_module_host_internal(modules)
    }

    /// Installs an App module and its dependencies with the provided dependencies config.
    pub fn install_app_with_dependencies<
        M: RegisteredModule
            + From<Contract<Chain>>
            + ExecutableContract
            + QueryableContract
            + ContractInstance<Chain>
            + DependencyCreation
            + InstallConfig,
    >(
        &self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
    ) -> AbstractClientResult<RemoteApplication<Chain, IBC, M>> {
        let mut install_configs: Vec<ModuleInstallConfig> =
            M::dependency_install_configs(dependencies_config)?;
        install_configs.push(M::install_config(module_configuration)?);

        self.install_module_host_internal(install_configs)
    }

    /// Upgrades the account to the latest version
    ///
    /// Migrates account to its respective new versions.
    /// Note that execution will be done through source chain
    pub fn upgrade(&self, version: ModuleVersion) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        let modules = vec![(
            ModuleInfo::from_id(abstract_std::constants::ACCOUNT, version.clone())?,
            Some(
                to_json_binary(&abstract_std::account::MigrateMsg {})
                    .map_err(Into::<CwOrchError>::into)?,
            ),
        )];
        self.ibc_client_execute(ibc_client::ExecuteMsg::RemoteAction {
            host_chain: self.host_chain_id(),
            action: ibc_host::HostAction::Dispatch {
                account_msgs: vec![account::ExecuteMsg::Upgrade { modules }],
            },
        })
    }

    /// Returns owner of the account
    pub fn ownership(&self) -> AbstractClientResult<ownership::Ownership<String>> {
        let account = self.address()?;
        self.host_chain()
            .query(&account::QueryMsg::Ownership {}, &account)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Returns the owner address of the account.
    /// If the account is a sub-account, it will return the top-level owner address.
    pub fn owner(&self) -> AbstractClientResult<Addr> {
        self.abstr_owner_account
            .top_level_owner()
            .map(|tlo| tlo.address)
            .map_err(Into::into)
    }

    /// Executes a [`CosmosMsg`] on the account.
    pub fn execute(
        &self,
        execute_msgs: impl IntoIterator<Item = impl Into<CosmosMsg>>,
    ) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        let msgs = execute_msgs.into_iter().map(Into::into).collect();
        self.execute_on_account(vec![abstract_std::account::ExecuteMsg::Execute { msgs }])
    }

    /// Executes a list of [account::ExecuteMsg] on the account.
    pub fn execute_on_account(
        &self,
        account_msgs: Vec<account::ExecuteMsg>,
    ) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        self.ibc_client_execute(ibc_client::ExecuteMsg::RemoteAction {
            host_chain: self.host_chain_id(),
            action: ibc_host::HostAction::Dispatch { account_msgs },
        })
    }

    /// Queries a module on the account.
    pub fn query_module<Q: Serialize + std::fmt::Debug, T: Serialize + DeserializeOwned>(
        &self,
        module_id: ModuleId,
        msg: &Q,
    ) -> AbstractClientResult<T> {
        let mut module_address_response = self.module_addresses(vec![module_id.to_owned()])?;
        let (_, module_addr) = module_address_response.modules.pop().unwrap();
        let response = self
            .host_chain()
            .query(msg, &module_addr)
            .map_err(Into::into)?;
        Ok(response)
    }

    /// Deposit funds to the account of the account with IBC transfer
    pub fn deposit(
        &self,
        funds: Vec<Coin>,
        memo: Option<String>,
    ) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        self.ibc_client_execute(ibc_client::ExecuteMsg::SendFunds {
            host_chain: self.host_chain_id(),
            funds,
            memo,
        })
    }

    /// Module infos of installed modules on account
    pub fn module_infos(&self) -> AbstractClientResult<ModuleInfosResponse> {
        let account = self.address()?;

        let mut module_infos: Vec<AccountModuleInfo> = vec![];
        loop {
            let last_module_id: Option<String> = module_infos
                .last()
                .map(|module_info| module_info.id.clone());
            let res: ModuleInfosResponse = self
                .host_chain()
                .query(
                    &account::QueryMsg::ModuleInfos {
                        start_after: last_module_id,
                        limit: None,
                    },
                    &account,
                )
                .map_err(Into::into)?;
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
        let account = self.address()?;

        self.host_chain()
            .query(&account::QueryMsg::ModuleAddresses { ids }, &account)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Check if module installed on account
    pub fn module_installed(&self, id: ModuleId) -> AbstractClientResult<bool> {
        let account = self.address()?;

        let key = account::state::ACCOUNT_MODULES.key(id).to_vec();
        let maybe_module_addr = self
            .host_chain()
            .wasm_querier()
            .raw_query(&account, key)
            .map_err(Into::into)?;
        Ok(!maybe_module_addr.is_empty())
    }

    /// Check if module installed on account
    pub fn ibc_status(&self) -> AbstractClientResult<bool> {
        self.module_installed(IBC_CLIENT)
    }

    /// Retrieve installed application on account
    pub fn application<
        M: RegisteredModule
            + From<Contract<Chain>>
            + ExecutableContract
            + QueryableContract
            + ContractInstance<Chain>,
    >(
        &self,
    ) -> AbstractClientResult<RemoteApplication<Chain, IBC, M>> {
        let module = self.module()?;

        RemoteApplication::new(self.clone(), module)
    }

    /// Install module on account
    fn install_module_host_internal<
        M: RegisteredModule
            + From<Contract<Chain>>
            + ExecutableContract
            + QueryableContract
            + ContractInstance<Chain>,
    >(
        &self,
        modules: Vec<ModuleInstallConfig>,
    ) -> AbstractClientResult<RemoteApplication<Chain, IBC, M>> {
        let _ = self.ibc_client_execute(ibc_client::ExecuteMsg::RemoteAction {
            host_chain: self.host_chain_id(),
            action: ibc_host::HostAction::Dispatch {
                account_msgs: vec![account::ExecuteMsg::InstallModules { modules }],
            },
        })?;

        let module = self.module()?;
        RemoteApplication::new(self.clone(), module)
    }

    pub(crate) fn host_abstract(&self) -> AbstractClientResult<Abstract<Chain>> {
        Abstract::load_from(self.host_chain.clone()).map_err(Into::into)
    }

    pub(crate) fn ibc_client_execute(
        &self,
        msg: ibc_client::ExecuteMsg,
    ) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        let exec_msg = to_json_binary(&msg).unwrap();
        let funds = if let ibc_client::ExecuteMsg::SendFunds { funds, .. } = msg {
            funds
        } else {
            vec![]
        };
        let msg = account::ExecuteMsg::ExecuteOnModule {
            module_id: IBC_CLIENT.to_owned(),
            exec_msg,
            funds,
        };

        let tx_response = self.abstr_owner_account.execute(&msg, &[])?;
        let packets = self
            .ibc_env
            .await_packets(&self.origin_chain().chain_id(), tx_response)
            .map_err(Into::into)?;
        packets.into_result()?;
        Ok(IbcTxAnalysisV2(packets))
    }

    pub(crate) fn module<T: RegisteredModule + From<Contract<Chain>>>(
        &self,
    ) -> AbstractClientResult<T> {
        let module_id = T::module_id();
        let account_module_id = T::installed_module_contract_id(&self.id());
        let maybe_module_addr = self.module_addresses(vec![module_id.to_string()])?.modules;

        if !maybe_module_addr.is_empty() {
            let contract = Contract::new(account_module_id, self.host_chain());
            contract.set_address(&maybe_module_addr[0].1);
            let module: T = contract.into();
            Ok(module)
        } else {
            Err(AbstractClientError::ModuleNotInstalled {})
        }
    }
}

impl<Chain: MutCwEnv + IbcQueryHandler, IBC: InterchainEnv<Chain>> RemoteAccount<Chain, IBC> {
    /// Set balance for the Account
    pub fn set_balance(&self, amount: &[Coin]) -> AbstractClientResult<()> {
        self.host_chain()
            .set_balance(&self.address()?, amount.to_vec())
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Add balance to the Account
    pub fn add_balance(&self, amount: &[Coin]) -> AbstractClientResult<()> {
        self.host_chain()
            .add_balance(&self.address()?, amount.to_vec())
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

impl<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> std::fmt::Display
    for RemoteAccount<Chain, IBC>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.remote_account_id)
    }
}

impl<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> std::fmt::Debug
    for RemoteAccount<Chain, IBC>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.remote_account_id)
    }
}
