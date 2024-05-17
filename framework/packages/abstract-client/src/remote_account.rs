use abstract_interface::{
    Abstract, AbstractAccount, DependencyCreation, InstallConfig, ManagerQueryFns as _,
    RegisteredModule, VCQueryFns,
};
use abstract_std::{
    ibc_client, ibc_host,
    manager::{
        self, state::AccountInfo, InfoResponse, ManagerModuleInfo, ModuleAddressesResponse,
        ModuleInfosResponse, ModuleInstallConfig,
    },
    objects::{
        chain_name::ChainName,
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion},
        nested_admin::MAX_ADMIN_RECURSION,
        AccountId,
    },
    proxy, IBC_CLIENT, PROXY,
};
use cosmwasm_std::{to_json_binary, Uint128};
use cw_orch::{environment::MutCwEnv, prelude::*};

use crate::{client::AbstractClientResult, AbstractClientError};

/// Represents an existing remote Abstract account.
///
/// TODO: update doc
/// Get this struct from [`AbstractClient::account_from_namespace`](crate::AbstractClient)
/// or create a new account with the [`AccountBuilder`].
pub struct RemoteAccount<Chain: CwEnv> {
    pub(crate) abstr_owner_account: AbstractAccount<Chain>,
    remote_account_id: AccountId,
    remote_abstract: Abstract<Chain>,
}

impl<Chain: CwEnv> RemoteAccount<Chain> {
    pub(crate) fn new(
        abstr_owner_account: AbstractAccount<Chain>,
        remote_account_id: AccountId,
        remote_abstract: Abstract<Chain>,
    ) -> Self {
        Self {
            abstr_owner_account,
            remote_account_id,
            remote_abstract,
        }
    }

    /// Get the [`AccountId`] of the Account
    pub fn id(&self) -> AccountId {
        self.remote_account_id.clone()
    }

    pub fn host_chain(&self) -> ChainName {
        ChainName::from_string(self.remote_chain().env_info().chain_name).unwrap()
    }

    // TODO:
    // pub fn deposit(
    //     &self,
    //     assets: Vec<AssetInfo>,
    // ) -> AbstractClientResult<<Chain as TxHandler>::Response> {
    // We try to batch it so if one of the deposits fail - we just fail tx
    // }

    fn remote_chain(&self) -> Chain {
        self.remote_abstract.version_control.get_chain().clone()
    }

    fn origin_chain(&self) -> Chain {
        self.abstr_owner_account.manager.get_chain().clone()
    }

    /// Get proxy address of the remote account
    pub fn proxy(&self) -> AbstractClientResult<Addr> {
        let base_response = self
            .remote_abstract
            .version_control
            .account_base(self.remote_account_id.clone())?;
        Ok(base_response.account_base.proxy)
    }

    /// Get manager address of the remote account
    pub fn manager(&self) -> AbstractClientResult<Addr> {
        let base_response = self
            .remote_abstract
            .version_control
            .account_base(self.remote_account_id.clone())?;
        Ok(base_response.account_base.manager)
    }

    /// Query account balance of a given denom
    // TODO: Asset balance?
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

    /// Query account info
    pub fn info(&self) -> AbstractClientResult<AccountInfo<Addr>> {
        let info_response: InfoResponse = self
            .remote_chain()
            .query(&manager::QueryMsg::Info {}, &self.manager()?)
            .map_err(Into::into)?;
        Ok(info_response.info)
    }

    /// Install an application on remote account.
    pub fn install_app<M: InstallConfig>(
        &self,
        configuration: &M::InitMsg,
    ) -> AbstractClientResult<Chain::Response> {
        let modules = vec![M::install_config(configuration)?];

        self.install_module_remote_internal(modules)
    }

    /// Install an adapter on remote account.
    pub fn install_adapter<M: InstallConfig<InitMsg = Empty>>(
        &self,
    ) -> AbstractClientResult<Chain::Response> {
        let modules = vec![M::install_config(&cosmwasm_std::Empty {})?];

        self.install_module_remote_internal(modules)
    }

    /// Installs an App module and its dependencies with the provided dependencies config.
    pub fn install_app_with_dependencies<M: DependencyCreation + InstallConfig>(
        &self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
    ) -> AbstractClientResult<Chain::Response> {
        let mut install_configs: Vec<ModuleInstallConfig> =
            M::dependency_install_configs(dependencies_config)?;
        install_configs.push(M::install_config(module_configuration)?);

        self.install_module_remote_internal(install_configs)
    }

    /// Upgrades the account to the latest version
    ///
    /// Migrates manager and proxy contracts to their respective new versions.
    pub fn upgrade(&self, version: ModuleVersion) -> AbstractClientResult<Chain::Response> {
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
    /// TODO: is it useful?
    pub fn ownership(&self) -> AbstractClientResult<cw_ownable::Ownership<String>> {
        let manager = self.manager()?;
        self.remote_chain()
            .query(&manager::QueryMsg::Ownership {}, &manager)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    /// Returns the owner address of the account.
    /// If the account is a sub-account, it will return the top-level owner address.
    pub fn owner(&self) -> AbstractClientResult<Addr> {
        let mut governance = self
            .abstr_owner_account
            .manager
            .info()?
            .info
            .governance_details;

        let environment = self.origin_chain();
        // Get sub-accounts until we get non-sub-account governance or reach recursion limit
        for _ in 0..MAX_ADMIN_RECURSION {
            match &governance {
                GovernanceDetails::SubAccount { manager, .. } => {
                    governance = environment
                        .query::<_, InfoResponse>(&manager::QueryMsg::Info {}, manager)
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

    // /// Executes a [`CosmosMsg`] on the proxy of the account.
    // pub fn execute(
    //     &self,
    //     execute_msgs: impl IntoIterator<Item = impl Into<CosmosMsg>>,
    //     funds: &[Coin],
    // ) -> AbstractClientResult<<Chain as TxHandler>::Response> {
    //     let msgs = execute_msgs.into_iter().map(Into::into).collect();
    //     self.abstr_account
    //         .manager
    //         .execute(
    //             &abstract_std::manager::ExecuteMsg::ExecOnModule {
    //                 module_id: PROXY.to_owned(),
    //                 exec_msg: to_json_binary(&abstract_std::proxy::ExecuteMsg::ModuleAction {
    //                     msgs,
    //                 })
    //                 .map_err(AbstractInterfaceError::from)?,
    //             },
    //             Some(funds),
    //         )
    //         .map_err(Into::into)
    // }

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

    /// Install module on remote account
    fn install_module_remote_internal(
        &self,
        modules: Vec<ModuleInstallConfig>,
    ) -> AbstractClientResult<Chain::Response> {
        self.ibc_client_execute(ibc_client::ExecuteMsg::RemoteAction {
            host_chain: self.host_chain(),
            action: ibc_host::HostAction::Dispatch {
                manager_msgs: vec![manager::ExecuteMsg::InstallModules { modules }],
            },
        })
    }

    fn ibc_client_addr(&self) -> AbstractClientResult<Addr> {
        self.abstr_owner_account
            .manager
            .module_address(IBC_CLIENT)
            .map_err(Into::into)
    }

    // TODO: redundant Serialize trait bound https://github.com/AbstractSDK/cw-orchestrator/pull/397
    // fn ibc_client_query<D: Serialize + DeserializeOwned>(
    //     &self,
    //     query: &ibc_client::QueryMsg,
    // ) -> AbstractClientResult<D> {
    //     let ibc_client_addr = self.ibc_client_addr()?;

    //     self.origin_chain()
    //         .query(query, &ibc_client_addr)
    //         .map_err(Into::into)
    //         .map_err(Into::into)
    // }

    fn ibc_client_execute(
        &self,
        exec_msg: ibc_client::ExecuteMsg,
    ) -> AbstractClientResult<Chain::Response> {
        // let ibc_client_addr = self.ibc_client_addr()?;

        // self.abstr_owner_account
        //     .manager
        //     .execute_on_module(IBC_CLIENT, exec_msg)
        // self.origin_chain()
        //     .execute(exec_msg, &[], &ibc_client_addr)
        // .map_err(Into::into)
        // .map_err(Into::into)

        let msg = proxy::ExecuteMsg::IbcAction { msg: exec_msg };

        self.abstr_owner_account
            .manager
            .execute_on_module(PROXY, msg)
            .map_err(Into::into)
    }
}

impl<Chain: MutCwEnv> RemoteAccount<Chain> {
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

// TODO:
// impl<Chain: CwEnv> Display for RemoteAccount<Chain> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.abstr_account)
//     }
// }

// impl<Chain: CwEnv> Debug for RemoteAccount<Chain> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.abstr_account)
//     }
// }
