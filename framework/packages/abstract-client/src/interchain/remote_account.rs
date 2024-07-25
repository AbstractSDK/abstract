//! # Represents Remote Abstract Account
//!
//! [`RemoteAccount`] allows you to interact with your or another user remote Abstract Account
//!

use abstract_interface::{
    Abstract, AbstractAccount, AbstractInterfaceError, AccountDetails, DependencyCreation,
    IbcClient, InstallConfig, ManagerQueryFns as _, RegisteredModule, VCQueryFns as _,
};
use abstract_std::{
    ibc_client::{self, QueryMsgFns as _},
    ibc_host,
    manager::{
        self, state::AccountInfo, InfoResponse, ManagerModuleInfo, ModuleAddressesResponse,
        ModuleInfosResponse, ModuleInstallConfig,
    },
    objects::{
        module::{ModuleId, ModuleInfo, ModuleVersion},
        namespace::Namespace,
        ownership, AccountId, AssetEntry, TruncatedChainId,
    },
    proxy, IBC_CLIENT, PROXY,
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
pub struct RemoteAccountBuilder<'a, Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> {
    pub(crate) ibc_env: &'a IBC,
    pub(crate) remote_chain: Chain,
    namespace: Option<Namespace>,
    base_asset: Option<AssetEntry>,
    owner_account: Account<Chain>,
    install_modules: Vec<ModuleInstallConfig>,
    // TODO: how we want to manage funds ibc-wise?
    // funds: AccountCreationFunds,
}

impl<'a, Chain: IbcQueryHandler> Account<Chain> {
    /// Builder for creating a new Abstract [`RemoteAccount`].
    pub fn remote_account_builder<IBC: InterchainEnv<Chain>>(
        &self,
        interchain_env: &'a IBC,
        remote_abstract: &AbstractClient<Chain>,
    ) -> RemoteAccountBuilder<'a, Chain, IBC> {
        RemoteAccountBuilder::new(self.clone(), interchain_env, remote_abstract.environment())
    }

    /// Get [`RemoteAccount`] of this account
    pub fn remote_account<IBC: InterchainEnv<Chain>>(
        &'a self,
        interchain_env: &'a IBC,
        remote_chain: Chain,
    ) -> AbstractClientResult<RemoteAccount<Chain, IBC>> {
        // Make sure ibc client installed on account
        let ibc_client = self.application::<IbcClient<Chain>>()?;
        let remote_chain_name = TruncatedChainId::from_chain_id(&remote_chain.chain_id());
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
                TruncatedChainId::from_chain_id(&owner_account.manager.environment().chain_id());
            id.push_chain(chain_name);
            id
        };

        Ok(RemoteAccount::new(
            owner_account,
            remote_account_id,
            remote_chain,
            interchain_env,
        ))
    }
}

impl<'a, Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> RemoteAccountBuilder<'a, Chain, IBC> {
    pub(crate) fn new(
        owner_account: Account<Chain>,
        ibc_env: &'a IBC,
        remote_chain: Chain,
    ) -> Self {
        Self {
            ibc_env,
            remote_chain,
            namespace: None,
            base_asset: None,
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

    /// Base Asset for the account
    pub fn base_asset(mut self, base_asset: AssetEntry) -> Self {
        self.base_asset = Some(base_asset);
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
    pub fn build(self) -> AbstractClientResult<RemoteAccount<'a, Chain, IBC>> {
        let remote_chain = self.remote_chain;
        let remote_env_info = remote_chain.env_info();

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
            base_asset: self.base_asset.clone(),
            install_modules,
            ..Default::default()
        };
        let host_chain = TruncatedChainId::from_chain_id(&remote_env_info.chain_id);

        let response = owner_account
            .abstr_account
            .create_remote_account(account_details, host_chain)?;
        self.ibc_env
            .await_and_check_packets(&env_info.chain_id, response)?;

        let remote_account_id = {
            let mut id = owner_account.id()?;
            let chain_name = TruncatedChainId::from_chain_id(
                &owner_account.abstr_account.manager.environment().chain_id(),
            );
            id.push_chain(chain_name);
            id
        };

        Ok(RemoteAccount::new(
            owner_account.abstr_account,
            remote_account_id,
            remote_chain,
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
pub struct RemoteAccount<'a, Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> {
    pub(crate) abstr_owner_account: AbstractAccount<Chain>,
    remote_account_id: AccountId,
    remote_chain: Chain,
    ibc_env: &'a IBC,
}

impl<'a, Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> RemoteAccount<'a, Chain, IBC> {
    pub(crate) fn new(
        abstr_owner_account: AbstractAccount<Chain>,
        remote_account_id: AccountId,
        remote_chain: Chain,
        ibc_env: &'a IBC,
    ) -> Self {
        Self {
            abstr_owner_account,
            remote_account_id,
            remote_chain,
            ibc_env,
        }
    }

    /// Get the [`AccountId`] of the Account
    pub fn id(&self) -> AccountId {
        self.remote_account_id.clone()
    }

    /// Truncated chain id of the host chain
    pub fn host_chain(&self) -> TruncatedChainId {
        TruncatedChainId::from_chain_id(&self.remote_chain().env_info().chain_id)
    }

    fn remote_chain(&self) -> Chain {
        self.remote_chain.clone()
    }

    fn origin_chain(&self) -> Chain {
        self.abstr_owner_account.manager.environment().clone()
    }

    /// Address of the proxy
    pub fn proxy(&self) -> AbstractClientResult<Addr> {
        let base_response = self
            .remote_abstract()?
            .version_control
            .account_base(self.remote_account_id.clone())?;
        Ok(base_response.account_base.proxy)
    }

    /// Address of the account (proxy)
    pub fn address(&self) -> AbstractClientResult<Addr> {
        self.proxy()
    }

    /// Get manager address of the account
    pub fn manager(&self) -> AbstractClientResult<Addr> {
        let base_response = self
            .remote_abstract()?
            .version_control
            .account_base(self.remote_account_id.clone())?;
        Ok(base_response.account_base.manager)
    }

    /// Query account balance of a given denom
    pub fn query_balance(&self, denom: impl Into<String>) -> AbstractClientResult<Uint128> {
        let coins = self
            .remote_chain()
            .bank_querier()
            .balance(self.proxy()?, Some(denom.into()))
            .map_err(Into::into)?;

        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    /// Query account balances of all denoms
    pub fn query_balances(&self) -> AbstractClientResult<Vec<Coin>> {
        self.remote_chain()
            .bank_querier()
            .balance(self.proxy()?, None)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Query account balance of a given denom
    pub fn query_ans_balance(&self, ans_asset: AssetEntry) -> AbstractClientResult<Uint128> {
        let proxy_addr = self.proxy()?;
        let holding_ammount: proxy::HoldingAmountResponse = self
            .remote_chain
            .query(
                &proxy::QueryMsg::HoldingAmount {
                    identifier: ans_asset,
                },
                &proxy_addr,
            )
            .map_err(Into::into)?;

        Ok(holding_ammount.amount)
    }

    /// Query account info
    pub fn info(&self) -> AbstractClientResult<AccountInfo> {
        let info_response: InfoResponse = self
            .remote_chain()
            .query(&manager::QueryMsg::Info {}, &self.manager()?)
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

        self.install_module_remote_internal(modules)
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

        self.install_module_remote_internal(modules)
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

        self.install_module_remote_internal(install_configs)
    }

    /// Upgrades the account to the latest version
    ///
    /// Migrates manager and proxy contracts to their respective new versions.
    /// Note that execution will be done through source chain
    pub fn upgrade(&self, version: ModuleVersion) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        let modules = vec![
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
        ];
        self.ibc_client_execute(ibc_client::ExecuteMsg::RemoteAction {
            host_chain: self.host_chain(),
            action: ibc_host::HostAction::Dispatch {
                manager_msgs: vec![manager::ExecuteMsg::Upgrade { modules }],
            },
        })
    }

    /// Returns owner of the account
    pub fn ownership(&self) -> AbstractClientResult<ownership::Ownership<String>> {
        let manager = self.manager()?;
        self.remote_chain()
            .query(&manager::QueryMsg::Ownership {}, &manager)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Returns the owner address of the account.
    /// If the account is a sub-account, it will return the top-level owner address.
    pub fn owner(&self) -> AbstractClientResult<Addr> {
        self.abstr_owner_account
            .manager
            .top_level_owner()
            .map(|tlo| tlo.address)
            .map_err(Into::into)
    }

    /// Executes a [`CosmosMsg`] on the proxy of the account.
    pub fn execute(
        &self,
        execute_msgs: impl IntoIterator<Item = impl Into<CosmosMsg>>,
    ) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        let msgs = execute_msgs.into_iter().map(Into::into).collect();
        self.execute_on_manager(vec![manager::ExecuteMsg::ExecOnModule {
            module_id: PROXY.to_owned(),
            exec_msg: to_json_binary(&abstract_std::proxy::ExecuteMsg::ModuleAction { msgs })
                .map_err(AbstractInterfaceError::from)?,
        }])
    }

    /// Executes a list of [manager::ExecuteMsg] on the manager of the account.
    pub fn execute_on_manager(
        &self,
        manager_msgs: Vec<manager::ExecuteMsg>,
    ) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        self.ibc_client_execute(ibc_client::ExecuteMsg::RemoteAction {
            host_chain: self.host_chain(),
            action: ibc_host::HostAction::Dispatch { manager_msgs },
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
            .remote_chain()
            .query(msg, &module_addr)
            .map_err(Into::into)?;
        Ok(response)
    }

    /// Deposit funds to the manager of the account with IBC transfer
    pub fn deposit(
        &self,
        funds: Vec<Coin>,
        memo: Option<String>,
    ) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        self.ibc_client_execute(ibc_client::ExecuteMsg::SendFunds {
            host_chain: self.host_chain(),
            funds,
            memo,
        })
    }

    /// Module infos of installed modules on account
    pub fn module_infos(&self) -> AbstractClientResult<ModuleInfosResponse> {
        let manager = self.manager()?;

        let mut module_infos: Vec<ManagerModuleInfo> = vec![];
        loop {
            let last_module_id: Option<String> = module_infos
                .last()
                .map(|module_info| module_info.id.clone());
            let res: ModuleInfosResponse = self
                .remote_chain()
                .query(
                    &manager::QueryMsg::ModuleInfos {
                        start_after: last_module_id,
                        limit: None,
                    },
                    &manager,
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
        let manager = self.manager()?;

        self.remote_chain()
            .query(&manager::QueryMsg::ModuleAddresses { ids }, &manager)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Check if module installed on account
    pub fn module_installed(&self, id: ModuleId) -> AbstractClientResult<bool> {
        let manager = self.manager()?;

        let key = manager::state::ACCOUNT_MODULES.key(id).to_vec();
        let maybe_module_addr = self
            .remote_chain()
            .wasm_querier()
            .raw_query(manager, key)
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
        &'a self,
    ) -> AbstractClientResult<RemoteApplication<'a, Chain, IBC, M>> {
        let module = self.module()?;

        RemoteApplication::new(self, module)
    }

    /// Install module on account
    fn install_module_remote_internal<
        M: RegisteredModule
            + From<Contract<Chain>>
            + ExecutableContract
            + QueryableContract
            + ContractInstance<Chain>,
    >(
        &'a self,
        modules: Vec<ModuleInstallConfig>,
    ) -> AbstractClientResult<RemoteApplication<'a, Chain, IBC, M>> {
        let _ = self.ibc_client_execute(ibc_client::ExecuteMsg::RemoteAction {
            host_chain: self.host_chain(),
            action: ibc_host::HostAction::Dispatch {
                manager_msgs: vec![manager::ExecuteMsg::InstallModules { modules }],
            },
        })?;

        let module = self.module()?;
        RemoteApplication::new(self, module)
    }

    pub(crate) fn remote_abstract(&self) -> AbstractClientResult<Abstract<Chain>> {
        Abstract::load_from(self.remote_chain.clone()).map_err(Into::into)
    }

    pub(crate) fn ibc_client_execute(
        &self,
        exec_msg: ibc_client::ExecuteMsg,
    ) -> AbstractClientResult<IbcTxAnalysisV2<Chain>> {
        let msg = proxy::ExecuteMsg::IbcAction { msg: exec_msg };

        let tx_response = self
            .abstr_owner_account
            .manager
            .execute_on_module(PROXY, msg)?;
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
            let contract = Contract::new(account_module_id, self.remote_chain());
            contract.set_address(&maybe_module_addr[0].1);
            let module: T = contract.into();
            Ok(module)
        } else {
            Err(AbstractClientError::ModuleNotInstalled {})
        }
    }
}

impl<'a, Chain: MutCwEnv + IbcQueryHandler, IBC: InterchainEnv<Chain>>
    RemoteAccount<'a, Chain, IBC>
{
    /// Set balance for the Proxy
    pub fn set_balance(&self, amount: &[Coin]) -> AbstractClientResult<()> {
        self.remote_chain()
            .set_balance(&self.proxy()?, amount.to_vec())
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Add balance to the Proxy
    pub fn add_balance(&self, amount: &[Coin]) -> AbstractClientResult<()> {
        self.remote_chain()
            .add_balance(&self.proxy()?, amount.to_vec())
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

impl<'a, Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> std::fmt::Display
    for RemoteAccount<'a, Chain, IBC>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.remote_account_id)
    }
}

impl<'a, Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>> std::fmt::Debug
    for RemoteAccount<'a, Chain, IBC>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.remote_account_id)
    }
}
