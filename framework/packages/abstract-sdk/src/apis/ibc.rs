//! # Ibc Client
//! The IbcClient object provides helper function for ibc-related queries or actions.
//!

use abstract_std::{
    base,
    ibc::{ModuleQuery},
    ibc_client::{self, ExecuteMsg as IbcClientMsg, InstalledModuleIdentification},
    ibc::Callback,
    ibc_host::HostAction,
    manager::ModuleInstallConfig,
    objects::module::{ModuleInfo, ModuleVersion},
    proxy::ExecuteMsg,
    IBC_CLIENT,
};
use cosmwasm_std::{to_json_binary, wasm_execute, Addr, Coin, CosmosMsg, Deps, QueryRequest};
use serde::Serialize;

use super::{AbstractApi, ApiIdentification};
use crate::{
    features::{AccountIdentification, ModuleIdentification},
    AbstractSdkResult, ModuleRegistryInterface,
};

/// Interact with other chains over IBC.
pub trait IbcInterface:
    AccountIdentification + ModuleRegistryInterface + ModuleIdentification
{
    /**
        API for interacting with the Abstract IBC client.

        # Example
        ```
        use abstract_sdk::prelude::*;
        # use cosmwasm_std::testing::mock_dependencies;
        # use abstract_sdk::mock_module::MockModule;
        # let module = MockModule::new();
        # let deps = mock_dependencies();

        let ibc_client: IbcClient<MockModule>  = module.ibc_client(deps.as_ref());
        ```
    */
    fn ibc_client<'a>(&'a self, deps: Deps<'a>) -> IbcClient<Self> {
        IbcClient { base: self, deps }
    }
}

impl<T> IbcInterface for T where
    T: AccountIdentification + ModuleRegistryInterface + ModuleIdentification
{
}

impl<'a, T: IbcInterface> AbstractApi<T> for IbcClient<'a, T> {
    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

impl<'a, T: IbcInterface> ApiIdentification for IbcClient<'a, T> {
    fn api_id() -> String {
        "IbcClient".to_owned()
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
    # let module = MockModule::new();
    # let deps = mock_dependencies();

    let ibc_client: IbcClient<MockModule>  = module.ibc_client(deps.as_ref());
    ```
*/
pub struct IbcClient<'a, T: IbcInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: IbcInterface> IbcClient<'a, T> {
    /// Get address of this module
    pub fn module_address(&self) -> AbstractSdkResult<Addr> {
        self.base
            .module_registry(self.deps)?
            // TODO: Update when client versions are fixed.
            // Use Dependencies trait bound
            .query_module(ModuleInfo::from_id_latest(IBC_CLIENT)?)?
            .reference
            .unwrap_native()
            .map_err(Into::into)
    }

    /// Registers the ibc client to be able to use IBC capabilities
    pub fn register_ibc_client(&self) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.manager_address(self.deps)?,
            &abstract_std::manager::ExecuteMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id(IBC_CLIENT, ModuleVersion::Latest)?,
                    None,
                )],
            },
            vec![],
        )?
        .into())
    }

    /// A simple helper to create and register a remote account
    pub fn create_remote_account(
        &self,
        // The chain on which you want to create an account
        host_chain: String,
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msg: abstract_std::ibc_client::ExecuteMsg::Register {
                    host_chain,
                    base_asset: None,
                    namespace: None,
                    install_modules: vec![],
                },
            },
            vec![],
        )?
        .into())
    }

    /// A simple helper to install an app on an account
    pub fn install_remote_app<M: Serialize>(
        &self,
        // The chain on which you want to install an app
        host_chain: String,
        module: ModuleInfo,
        init_msg: &M,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.host_action(
            host_chain,
            HostAction::Dispatch {
                manager_msgs: vec![abstract_std::manager::ExecuteMsg::InstallModules {
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
        host_chain: String,
        module: ModuleInfo,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.host_action(
            host_chain,
            HostAction::Dispatch {
                manager_msgs: vec![abstract_std::manager::ExecuteMsg::InstallModules {
                    modules: vec![ModuleInstallConfig::new(module, None)],
                }],
            },
        )
    }

    /// A simple helper to execute on a module
    pub fn execute_on_module<M: Serialize>(
        &self,
        host_chain: String,
        module_id: String,
        exec_msg: &M,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.host_action(
            host_chain,
            HostAction::Dispatch {
                manager_msgs: vec![abstract_std::manager::ExecuteMsg::ExecOnModule {
                    module_id,
                    exec_msg: to_json_binary(exec_msg)?,
                }],
            },
        )
    }

    /// Send module action from this module to the target module
    pub fn module_ibc_action<M: Serialize>(
        &self,
        host_chain: String,
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
        host_chain: String,
        target_module: InstalledModuleIdentification,
        query_msg: &base::QueryMsg<B, M>,
        callback_info: CallbackInfo,
    ) -> AbstractSdkResult<CosmosMsg> {
        let ibc_client_addr = self.module_address()?;
        let msg = wasm_execute(
            ibc_client_addr,
            &ibc_client::ExecuteMsg::IbcQuery {
                host_chain,
                query: QueryRequest::Custom(ModuleQuery {
                    target_module,
                    msg: to_json_binary(query_msg)?,
                }),
                callback_info,
            },
            vec![],
        )?;
        Ok(msg.into())
    }

    /// Send query from this module to the host chain
    pub fn ibc_query(
        &self,
        host_chain: String,
        query: impl Into<QueryRequest<ModuleQuery>>,
        callback: Callback,
    ) -> AbstractSdkResult<CosmosMsg> {
        let ibc_client_addr = self.module_address()?;
        let msg = wasm_execute(
            ibc_client_addr,
            &ibc_client::ExecuteMsg::IbcQuery {
                host_chain,
                queries: vec![query],
                callback,
            },
            vec![],
        )?;
        Ok(msg.into())
    }

    /// Send queries from this module to the host chain
    pub fn ibc_queries(
        &self,
        host_chain: String,
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

    /// Call a [`HostAction`] on the host of the provided `host_chain`.
    pub fn host_action(
        &self,
        host_chain: String,
        action: HostAction,
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msg: IbcClientMsg::RemoteAction { host_chain, action },
            },
            vec![],
        )?
        .into())
    }

    /// IbcClient the provided coins from the Account to its proxy on the `receiving_chain`.
    pub fn ics20_transfer(
        &self,
        host_chain: String,
        funds: Vec<Coin>,
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msg: IbcClientMsg::SendFunds { host_chain, funds },
            },
            vec![],
        )?
        .into())
    }

    /// Address of the remote proxy
    /// Note: only Accounts that are remote to *this* chain are queryable
    pub fn remote_proxy_addr(&self, host_chain: &str) -> AbstractSdkResult<Option<String>> {
        let account_id = self.base.account_id(self.deps)?;
        let ibc_client_addr = self.module_address()?;

        let (trace, sequence) = account_id.decompose();
        ibc_client::state::ACCOUNTS
            .query(
                &self.deps.querier,
                ibc_client_addr,
                (&trace, sequence, &host_chain.parse()?),
            )
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod test {
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    use super::*;
    use crate::mock_module::*;
    const TEST_HOST_CHAIN: &str = "hostchain";

    /// Tests that a host_action can be built with no callback
    #[test]
    fn test_host_action_no_callback() {
        let deps = mock_dependencies();
        let stub = MockModule::new();
        let client = stub.ibc_client(deps.as_ref());
        let msg = client.host_action(
            TEST_HOST_CHAIN.into(),
            HostAction::Dispatch {
                manager_msgs: vec![abstract_std::manager::ExecuteMsg::UpdateStatus {
                    is_suspended: None,
                }],
            },
        );
        assert_that!(msg).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_json_binary(&ExecuteMsg::IbcAction {
                msg: IbcClientMsg::RemoteAction {
                    host_chain: TEST_HOST_CHAIN.into(),
                    action: HostAction::Dispatch {
                        manager_msgs: vec![abstract_std::manager::ExecuteMsg::UpdateStatus {
                            is_suspended: None,
                        }],
                    },
                },
            })
            .unwrap(),
            funds: vec![],
        });
        assert_that!(msg.unwrap()).is_equal_to::<CosmosMsg>(expected);
    }

    /// Tests that the ics_20 transfer can be built and that the funds are passed into the sendFunds message not the execute message
    #[test]
    fn test_ics20_transfer() {
        let deps = mock_dependencies();
        let stub = MockModule::new();
        let client = stub.ibc_client(deps.as_ref());

        let expected_funds = coins(100, "denom");

        let msg = client.ics20_transfer(TEST_HOST_CHAIN.to_string(), expected_funds.clone());
        assert_that!(msg).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_json_binary(&ExecuteMsg::IbcAction {
                msg: IbcClientMsg::SendFunds {
                    host_chain: TEST_HOST_CHAIN.into(),
                    funds: expected_funds,
                },
            })
            .unwrap(),
            // ensure empty
            funds: vec![],
        });
        assert_that!(msg.unwrap()).is_equal_to::<CosmosMsg>(expected);
    }
}
