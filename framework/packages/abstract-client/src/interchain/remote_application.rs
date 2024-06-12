//! # Represents Abstract Remote Application
//!
//! [`RemoteApplication`] represents a module installed on a remote account

use std::fmt::Debug;

use abstract_interface::{AbstractInterfaceError, RegisteredModule};
use abstract_std::{adapter, ibc_client, ibc_host, manager};
use cosmwasm_std::to_json_binary;
use cw_orch::{contract::Contract, prelude::*};
use cw_orch_interchain::{IbcQueryHandler, InterchainEnv};
use serde::{de::DeserializeOwned, Serialize};

use crate::{client::AbstractClientResult, remote_account::RemoteAccount};

/// An application represents a module installed on a [`RemoteAccount`].
pub struct RemoteApplication<'a, Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>, M> {
    remote_account: &'a RemoteAccount<'a, Chain, IBC>,
    module: M,
}

impl<
        'a,
        Chain: IbcQueryHandler,
        IBC: InterchainEnv<Chain>,
        M: RegisteredModule + ExecutableContract + QueryableContract + ContractInstance<Chain>,
    > RemoteApplication<'a, Chain, IBC, M>
{
    /// Get module interface installed on provided remote account
    pub(crate) fn new(
        account: &'a RemoteAccount<'a, Chain, IBC>,
        module: M,
    ) -> AbstractClientResult<Self> {
        // Sanity check: the module must be installed on the account
        account.module_addresses(vec![M::module_id().to_string()])?;
        Ok(Self {
            remote_account: account,
            module,
        })
    }

    /// remote-account on which application is installed
    pub fn account(&self) -> &RemoteAccount<Chain, IBC> {
        self.remote_account
    }

    /// Execute message on application
    pub fn execute(&self, execute: &M::ExecuteMsg) -> AbstractClientResult<()> {
        self.remote_account
            .ibc_client_execute(ibc_client::ExecuteMsg::RemoteAction {
                host_chain: self.remote_account.host_chain(),
                action: ibc_host::HostAction::Dispatch {
                    manager_msgs: vec![manager::ExecuteMsg::ExecOnModule {
                        module_id: M::module_id().to_owned(),
                        exec_msg: to_json_binary(execute).map_err(AbstractInterfaceError::from)?,
                    }],
                },
            })
    }

    /// Queries request on  application
    pub fn query<G: DeserializeOwned + Serialize + Debug>(
        &self,
        query: &M::QueryMsg,
    ) -> AbstractClientResult<G> {
        self.module.query(query).map_err(Into::into)
    }

    /// Attempts to get a module on the application. This would typically be a dependency of the
    /// module of type `M`.
    pub fn module<T: RegisteredModule + From<Contract<Chain>>>(&self) -> AbstractClientResult<T> {
        self.remote_account.module()
    }

    /// Address of the module
    pub fn address(&self) -> AbstractClientResult<Addr> {
        self.module.address().map_err(Into::into)
    }
}

impl<'a, Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>, M: ContractInstance<Chain>>
    RemoteApplication<'a, Chain, IBC, M>
{
    /// Authorize this application on installed adapters. Accepts Module Id's of adapters
    pub fn authorize_on_adapters(&self, adapter_ids: &[&str]) -> AbstractClientResult<()> {
        let mut manager_msgs = vec![];
        for module_id in adapter_ids {
            manager_msgs.push(manager::ExecuteMsg::ExecOnModule {
                module_id: module_id.to_string(),
                exec_msg: to_json_binary(&adapter::ExecuteMsg::<Empty>::Base(
                    adapter::BaseExecuteMsg {
                        proxy_address: None,
                        msg: adapter::AdapterBaseMsg::UpdateAuthorizedAddresses {
                            to_add: vec![],
                            to_remove: vec![],
                        },
                    },
                ))
                .map_err(Into::<CwOrchError>::into)?,
            })
        }

        self.remote_account
            .ibc_client_execute(ibc_client::ExecuteMsg::RemoteAction {
                host_chain: self.remote_account.host_chain(),
                action: ibc_host::HostAction::Dispatch { manager_msgs },
            })?;
        Ok(())
    }
}
