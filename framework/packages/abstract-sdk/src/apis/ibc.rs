//! # Ibc Client
//! The IbcClient object provides helper function for ibc-related queries or actions.
//!

use super::{AbstractApi, ApiIdentification};
use crate::{
    features::{AccountIdentification, ModuleIdentification},
    AbstractSdkResult, ModuleInterface,
};
use abstract_core::{
    ibc::CallbackInfo,
    ibc_client::ExecuteMsg as IbcClientMsg,
    ibc_host::HostAction,
    manager::ModuleInstallConfig,
    objects::module::{ModuleInfo, ModuleVersion},
    proxy::ExecuteMsg,
    IBC_CLIENT,
};
use cosmwasm_std::{to_json_binary, wasm_execute, Coin, CosmosMsg, Deps};
use serde::Serialize;

/// Interact with other chains over IBC.
pub trait IbcInterface: AccountIdentification + ModuleInterface + ModuleIdentification {
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

impl<T> IbcInterface for T where T: AccountIdentification + ModuleInterface + ModuleIdentification {}

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
    /// Registers the ibc client to be able to use IBC capabilities
    pub fn register_ibc_client(&self) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.manager_address(self.deps)?,
            &abstract_core::manager::ExecuteMsg::InstallModules {
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
        host_chain: String, // The chain on which you want to create an account
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msgs: vec![abstract_core::ibc_client::ExecuteMsg::Register {
                    host_chain,
                    base_asset: None,
                    namespace: None,
                    install_modules: vec![],
                }],
            },
            vec![],
        )?
        .into())
    }

    /// A simple helper to install an app on an account
    pub fn install_remote_app<M: Serialize>(
        &self,
        host_chain: String, // The chain on which you want to create an account,
        module: ModuleInfo,
        init_msg: &M,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.host_action(
            host_chain,
            HostAction::Dispatch {
                manager_msg: abstract_core::manager::ExecuteMsg::InstallModules {
                    modules: vec![ModuleInstallConfig::new(
                        module,
                        Some(to_json_binary(&init_msg)?),
                    )],
                },
            },
            None,
        )
    }

    /// A simple helper install a remote api Module providing only the chain name
    pub fn install_remote_api<M: Serialize>(
        &self,
        host_chain: String, // The chain on which you want to create an account,
        module: ModuleInfo,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.host_action(
            host_chain,
            HostAction::Dispatch {
                manager_msg: abstract_core::manager::ExecuteMsg::InstallModules {
                    modules: vec![ModuleInstallConfig::new(module, None)],
                },
            },
            None,
        )
    }

    /// A simple helper to execute on a module
    pub fn execute_on_module<M: Serialize>(
        &self,
        host_chain: String, // The chain on which you want to create an account,
        module_id: String,
        exec_msg: &M,
    ) -> AbstractSdkResult<CosmosMsg> {
        self.host_action(
            host_chain,
            HostAction::Dispatch {
                manager_msg: abstract_core::manager::ExecuteMsg::ExecOnModule {
                    module_id,
                    exec_msg: to_json_binary(exec_msg)?,
                },
            },
            None,
        )
    }
    /// Call a [`HostAction`] on the host of the provided `host_chain`.
    pub fn host_action(
        &self,
        host_chain: String,
        action: HostAction,
        callback: Option<CallbackInfo>,
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::RemoteAction {
                    host_chain,
                    action,
                    callback_info: callback,
                }],
            },
            vec![],
        )?
        .into())
    }
    /// IbcClient the provided coins from the Account to its proxy on the `receiving_chain`.
    pub fn ics20_transfer(
        &self,
        receiving_chain: String,
        funds: Vec<Coin>,
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendFunds {
                    host_chain: receiving_chain,
                    funds,
                }],
            },
            vec![],
        )?
        .into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock_module::*;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;
    const TEST_HOST_CHAIN: &str = "host_chain";

    /// Tests that a host_action can be built with no callback
    #[test]
    fn test_host_action_no_callback() {
        let deps = mock_dependencies();
        let stub = MockModule::new();
        let client = stub.ibc_client(deps.as_ref());
        let msg = client.host_action(
            TEST_HOST_CHAIN.into(),
            HostAction::Dispatch {
                manager_msg: abstract_core::manager::ExecuteMsg::UpdateStatus {
                    is_suspended: None,
                },
            },
            None,
        );
        assert_that!(msg).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_json_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::RemoteAction {
                    host_chain: TEST_HOST_CHAIN.into(),
                    action: HostAction::Dispatch {
                        manager_msg: abstract_core::manager::ExecuteMsg::UpdateStatus {
                            is_suspended: None,
                        },
                    },
                    callback_info: None,
                }],
            })
            .unwrap(),
            funds: vec![],
        });
        assert_that!(msg.unwrap()).is_equal_to::<CosmosMsg>(expected);
    }

    /// Tests that a host_action can be built with a callback with more retries
    #[test]
    fn test_host_action_with_callback() {
        let deps = mock_dependencies();
        let stub = MockModule::new();
        let client = stub.ibc_client(deps.as_ref());

        let expected_callback = CallbackInfo {
            receiver: "callback_receiver".to_string(),
            id: "callback_id".to_string(),
            msg: None,
        };

        let actual = client.host_action(
            TEST_HOST_CHAIN.into(),
            HostAction::Dispatch {
                manager_msg: abstract_core::manager::ExecuteMsg::UpdateStatus {
                    is_suspended: None,
                },
            },
            Some(expected_callback.clone()),
        );

        assert_that!(actual).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_json_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::RemoteAction {
                    host_chain: TEST_HOST_CHAIN.into(),
                    action: HostAction::Dispatch {
                        manager_msg: abstract_core::manager::ExecuteMsg::UpdateStatus {
                            is_suspended: None,
                        },
                    },
                    callback_info: Some(expected_callback),
                }],
            })
            .unwrap(),
            funds: vec![],
        });

        assert_that!(actual.unwrap()).is_equal_to::<CosmosMsg>(expected);
    }

    /// Tests that the ics_20 transfer can be built and that the funds are passed into the sendFunds message not the execute message
    #[test]
    fn test_ics20_transfer() {
        let deps = mock_dependencies();
        let stub = MockModule::new();
        let client = stub.ibc_client(deps.as_ref());

        let expected_funds = coins(100, "denom");

        let msg = client.ics20_transfer(TEST_HOST_CHAIN.into(), expected_funds.clone());
        assert_that!(msg).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_json_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendFunds {
                    host_chain: TEST_HOST_CHAIN.into(),
                    funds: expected_funds,
                }],
            })
            .unwrap(),
            // ensure empty
            funds: vec![],
        });
        assert_that!(msg.unwrap()).is_equal_to::<CosmosMsg>(expected);
    }
}
