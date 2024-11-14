//! # Ibc Client
//! The IbcClient object provides helper function for ibc-related queries or actions.
//!

use abstract_std::{
    account::{ExecuteMsg, ModuleInstallConfig},
    base,
    ibc::{Callback, ModuleQuery, PACKET_LIFETIME},
    ibc_client::{self, ExecuteMsg as IbcClientMsg, InstalledModuleIdentification},
    ibc_host::{self, HostAction},
    objects::{module::ModuleInfo, ChannelEntry, TruncatedChainId},
    ABSTRACT_VERSION, IBC_CLIENT, ICS20,
};
use cosmwasm_std::{
    to_json_binary, wasm_execute, Addr, Coin, CosmosMsg, Deps, Empty, Env, IbcMsg, IbcTimeout,
    QueryRequest, SubMsg,
};
use serde::Serialize;

use super::AbstractApi;
use crate::{
    features::{AbstractNameService, AccountExecutor, AccountIdentification, ModuleIdentification},
    AbstractSdkResult, HookMemoBuilder, ModuleInterface, ModuleRegistryInterface,
};

/// Interact with other chains over IBC.
pub trait IbcInterface:
    AccountIdentification + ModuleRegistryInterface + ModuleIdentification + ModuleInterface
{
    /**
        API for interacting with the Abstract IBC client.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # use abstract_testing::prelude::*;
        # let deps = mock_dependencies();
        # let env = mock_env_validated(deps.api);
        # let account = admin_account(deps.api);
        # let module = MockModule::new(deps.api, account);

        let ibc_client: IbcClient<MockModule>  = module.ibc_client(deps.as_ref(), &env);
        ```
    */
    fn ibc_client<'a>(&'a self, deps: Deps<'a>, env: &'a Env) -> IbcClient<Self> {
        IbcClient {
            base: self,
            deps,
            env,
        }
    }
}

impl<T> IbcInterface for T where
    T: AccountIdentification + ModuleRegistryInterface + ModuleIdentification + ModuleInterface
{
}

impl<'a, T: IbcInterface> AbstractApi<T> for IbcClient<'a, T> {
    const API_ID: &'static str = "IbcClient";

    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

#[derive(Clone)]
/**
    API for interacting with the Abstract IBC client.

    # Example
    ```
    use abstract_sdk::prelude::*;
    # use cosmwasm_std::testing::mock_dependencies;
    # use abstract_sdk::mock_module::MockModule;
    # use abstract_testing::prelude::*;
    # let deps = mock_dependencies();
    # let env = mock_env_validated(deps.api);
    # let account = admin_account(deps.api);
    # let module = MockModule::new(deps.api, account);

    let ibc_client: IbcClient<MockModule>  = module.ibc_client(deps.as_ref(), &env);
    ```
*/
pub struct IbcClient<'a, T: IbcInterface> {
    base: &'a T,
    deps: Deps<'a>,
    env: &'a Env,
}

impl<'a, T: IbcInterface> IbcClient<'a, T> {
    /// Get address of this module
    pub fn module_address(&self) -> AbstractSdkResult<Addr> {
        let modules = self.base.modules(self.deps);
        modules.assert_module_dependency(IBC_CLIENT)?;
        self.base
            .module_registry(self.deps, self.env)?
            .query_module(ModuleInfo::from_id(IBC_CLIENT, ABSTRACT_VERSION.into())?)?
            .reference
            .unwrap_native()
            .map_err(Into::into)
    }

    /// Send module action from this module to the target module
    pub fn module_ibc_action<M: Serialize>(
        &self,
        host_chain: TruncatedChainId,
        target_module: ModuleInfo,
        exec_msg: &M,
        callback: Option<Callback>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let ibc_client_addr = self.module_address()?;
        let msg = wasm_execute(
            ibc_client_addr,
            &ibc_client::ExecuteMsg::ModuleIbcAction {
                host_chain,
                target_module,
                msg: to_json_binary(exec_msg)?,
                callback,
            },
            vec![],
        )?;
        Ok(msg.into())
    }

    /// Send module query from this module to the target module
    /// Use [`abstract_std::ibc::IbcResponseMsg::module_query_response`] to parse response
    pub fn module_ibc_query<B: Serialize, M: Serialize>(
        &self,
        host_chain: TruncatedChainId,
        target_module: InstalledModuleIdentification,
        query_msg: &base::QueryMsg<B, M>,
        callback: Callback,
    ) -> AbstractSdkResult<CosmosMsg> {
        let ibc_client_addr = self.module_address()?;
        let msg = wasm_execute(
            ibc_client_addr,
            &ibc_client::ExecuteMsg::IbcQuery {
                host_chain,
                queries: vec![QueryRequest::Custom(ModuleQuery {
                    target_module,
                    msg: to_json_binary(query_msg)?,
                })],
                callback,
            },
            vec![],
        )?;
        Ok(msg.into())
    }

    /// Send query from this module to the host chain
    pub fn ibc_query(
        &self,
        host_chain: TruncatedChainId,
        query: impl Into<QueryRequest<ModuleQuery>>,
        callback: Callback,
    ) -> AbstractSdkResult<CosmosMsg> {
        let ibc_client_addr = self.module_address()?;
        let msg = wasm_execute(
            ibc_client_addr,
            &ibc_client::ExecuteMsg::IbcQuery {
                host_chain,
                queries: vec![query.into()],
                callback,
            },
            vec![],
        )?;
        Ok(msg.into())
    }

    /// Send queries from this module to the host chain
    pub fn ibc_queries(
        &self,
        host_chain: TruncatedChainId,
        queries: Vec<QueryRequest<ModuleQuery>>,
        callback: Callback,
    ) -> AbstractSdkResult<CosmosMsg> {
        let ibc_client_addr = self.module_address()?;
        let msg = wasm_execute(
            ibc_client_addr,
            &ibc_client::ExecuteMsg::IbcQuery {
                host_chain,
                queries,
                callback,
            },
            vec![],
        )?;
        Ok(msg.into())
    }

    /// Address of the remote account
    /// Note: only Accounts that are remote to *this* chain are queryable
    pub fn remote_account_addr(
        &self,
        host_chain: &TruncatedChainId,
    ) -> AbstractSdkResult<Option<String>> {
        let ibc_client_addr = self.module_address()?;
        let account_id = self.base.account_id(self.deps)?;

        let (trace, sequence) = account_id.decompose();
        ibc_client::state::ACCOUNTS
            .query(
                &self.deps.querier,
                ibc_client_addr,
                (&trace, sequence, host_chain),
            )
            .map_err(Into::into)
    }
}

impl<'a, T: IbcInterface + AccountExecutor> IbcClient<'a, T> {
    /// Execute on ibc client
    pub fn execute(
        &self,
        msg: &abstract_std::ibc_client::ExecuteMsg,
        funds: Vec<Coin>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let wasm_msg = wasm_execute(
            self.base.account(self.deps)?.into_addr().into_string(),
            &ExecuteMsg::ExecuteOnModule::<Empty> {
                module_id: IBC_CLIENT.to_owned(),
                exec_msg: to_json_binary(&msg)?,
                funds,
            },
            vec![],
        )?;
        Ok(wasm_msg.into())
    }
    /// A simple helper to create and register a remote account
    pub fn create_remote_account(
        &self,
        // The chain on which you want to create an account
        host_chain: TruncatedChainId,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.execute(
            &abstract_std::ibc_client::ExecuteMsg::Register {
                host_chain,
                namespace: None,
                install_modules: vec![],
            },
            vec![],
        )
    }

    /// Call a [`HostAction`] on the host of the provided `host_chain`.
    pub fn host_action(
        &self,
        host_chain: TruncatedChainId,
        action: HostAction,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.execute(&IbcClientMsg::RemoteAction { host_chain, action }, vec![])
    }

    /// IbcClient the provided coins from the Account to its account on the `receiving_chain`.
    pub fn ics20_transfer(
        &self,
        host_chain: TruncatedChainId,
        funds: Vec<Coin>,
        memo: Option<String>,
        receiver: Option<String>,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.execute(
            &IbcClientMsg::SendFunds {
                host_chain,
                memo,
                receiver,
            },
            funds,
        )
    }

    /// A simple helper to install an app on an account
    pub fn install_remote_app<M: Serialize>(
        &self,
        // The chain on which you want to install an app
        host_chain: TruncatedChainId,
        module: ModuleInfo,
        init_msg: &M,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.host_action(
            host_chain,
            HostAction::Dispatch {
                account_msgs: vec![abstract_std::account::ExecuteMsg::InstallModules {
                    modules: vec![ModuleInstallConfig::new(
                        module,
                        Some(to_json_binary(&init_msg)?),
                    )],
                }],
            },
        )
    }

    /// A simple helper install a remote api Module providing only the chain name
    pub fn install_remote_api<M: Serialize>(
        &self,
        // The chain on which you want to install an api
        host_chain: TruncatedChainId,
        module: ModuleInfo,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.host_action(
            host_chain,
            HostAction::Dispatch {
                account_msgs: vec![abstract_std::account::ExecuteMsg::InstallModules {
                    modules: vec![ModuleInstallConfig::new(module, None)],
                }],
            },
        )
    }

    /// A simple helper to execute on a module
    /// Executes the message as the Account of the remote account
    /// I.e. can be used to execute admin actions on remote modules.
    pub fn execute_on_module<M: Serialize>(
        &self,
        host_chain: TruncatedChainId,
        module_id: String,
        exec_msg: &M,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.host_action(
            host_chain,
            HostAction::Dispatch {
                account_msgs: vec![abstract_std::account::ExecuteMsg::ExecuteOnModule {
                    module_id,
                    exec_msg: to_json_binary(exec_msg)?,
                    funds: vec![],
                }],
            },
        )
    }

    /// Address of the remote account
    /// Note: only works if account is local
    pub fn remote_account(
        &self,
        host_chain: &TruncatedChainId,
    ) -> AbstractSdkResult<Option<String>> {
        let account_id = self.base.account_id(self.deps)?;
        let ibc_client_addr = self.module_address()?;

        let (trace, sequence) = account_id.decompose();
        ibc_client::state::ACCOUNTS
            .query(
                &self.deps.querier,
                ibc_client_addr,
                (&trace, sequence, host_chain),
            )
            .map_err(Into::into)
    }
}

#[cfg(feature = "stargate")]
impl<'a, T: IbcInterface + AccountExecutor + AbstractNameService> IbcClient<'a, T> {
    /// Send funds from account to remote account with callback to the module
    ///
    /// This method should be combined with `.with_ics20_callback_reply` handler on a AbstractContract module object and use same reply_id
    /// Callback can be retrieved with `module.load_ics20_callback`
    ///
    /// Note: Payload occupied for saving callback and shouldn't be edited
    pub fn send_funds_with_callback(
        &self,
        host_chain: TruncatedChainId,
        funds: Coin,
        callback: Callback,
        reply_id: u64,
    ) -> AbstractSdkResult<SubMsg> {
        let name_service = self.base.name_service(self.deps(), &self.env);
        let ics20_channel_entry = ChannelEntry {
            connected_chain: host_chain.clone(),
            protocol: ICS20.to_string(),
        };
        let ics20_channel_id = name_service.query(&ics20_channel_entry)?;
        let payload = to_json_binary(&abstract_std::ibc::ICS20CallbackPayload {
            channel_id: ics20_channel_id.clone(),
            callback,
        })?;

        let ibc_client_addr = self.module_address()?;
        let remote_host = ibc_client::state::IBC_INFRA
            .query(&self.deps.querier, ibc_client_addr, &host_chain)?
            .ok_or(cosmwasm_std::StdError::generic_err(format!(
                "chain {host_chain} not registered in ibc_client"
            )))?;

        // Hook for sending the funds correctly to the sender
        let account_id = self.base.account_id(self.deps)?;
        let action_memo = HookMemoBuilder::new(
            remote_host.remote_abstract_host.clone(),
            &ibc_host::ExecuteMsg::Fund {
                src_account: account_id,
                src_chain: TruncatedChainId::from_chain_id(&self.env.block.chain_id),
            },
        )
        .callback(&self.env)
        .build()?;

        let transfer_msg = IbcMsg::Transfer {
            channel_id: ics20_channel_id,
            to_address: remote_host.remote_abstract_host,
            amount: funds,
            timeout: IbcTimeout::with_timestamp(self.env.block.time.plus_seconds(PACKET_LIFETIME)),
            memo: Some(action_memo),
        };
        Ok(SubMsg::reply_on_success(transfer_msg, reply_id).with_payload(payload))
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_testing::prelude::*;
    use cosmwasm_std::*;

    use super::*;
    use crate::{apis::traits::test::abstract_api_test, mock_module::*};
    const TEST_HOST_CHAIN: &str = "hostchain";

    /// Tests that a host_action can be built with no callback
    #[coverage_helper::test]
    fn test_host_action_no_callback() {
        let (deps, _, stub) = mock_module_setup();
        let env = mock_env_validated(deps.api);

        let client = stub.ibc_client(deps.as_ref(), &env);
        let msg = client.host_action(
            TEST_HOST_CHAIN.parse().unwrap(),
            HostAction::Dispatch {
                account_msgs: vec![abstract_std::account::ExecuteMsg::UpdateStatus {
                    is_suspended: None,
                }],
            },
        );
        assert!(msg.is_ok());

        let base = test_account(deps.api);
        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: base.addr().to_string(),
            msg: to_json_binary(&ExecuteMsg::ExecuteOnModule::<cosmwasm_std::Empty> {
                module_id: IBC_CLIENT.to_owned(),
                exec_msg: to_json_binary(&IbcClientMsg::RemoteAction {
                    host_chain: TEST_HOST_CHAIN.parse().unwrap(),
                    action: HostAction::Dispatch {
                        account_msgs: vec![abstract_std::account::ExecuteMsg::UpdateStatus {
                            is_suspended: None,
                        }],
                    },
                })
                .unwrap(),
                funds: vec![],
            })
            .unwrap(),
            funds: vec![],
        });
        assert_eq!(msg, Ok(expected));
    }

    /// Tests that the ics_20 transfer can be built and that the funds are passed into the sendFunds message not the execute message
    #[coverage_helper::test]
    fn test_ics20_transfer() {
        let (deps, _, stub) = mock_module_setup();
        let env = mock_env_validated(deps.api);

        let client = stub.ibc_client(deps.as_ref(), &env);

        let expected_funds = coins(100, "denom");

        let msg = client.ics20_transfer(
            TEST_HOST_CHAIN.parse().unwrap(),
            expected_funds.clone(),
            None,
            None,
        );
        assert!(msg.is_ok());

        let base = test_account(deps.api);
        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: base.addr().to_string(),
            msg: to_json_binary(&ExecuteMsg::ExecuteOnModule::<cosmwasm_std::Empty> {
                module_id: IBC_CLIENT.to_owned(),
                exec_msg: to_json_binary(&IbcClientMsg::SendFunds {
                    host_chain: TEST_HOST_CHAIN.parse().unwrap(),
                    memo: None,
                    receiver: None,
                })
                .unwrap(),
                funds: expected_funds,
            })
            .unwrap(),
            // ensure empty
            funds: vec![],
        });
        assert_eq!(msg, Ok(expected));
    }

    #[coverage_helper::test]
    fn abstract_api() {
        let (deps, _, app) = mock_module_setup();
        let env = mock_env_validated(deps.api);
        let client = app.ibc_client(deps.as_ref(), &env);

        abstract_api_test(client);
    }
}
