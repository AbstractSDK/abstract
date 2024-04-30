//! # Ibc Client
//! The IbcClient object provides helper function for ibc-related queries or actions.
//!

use abstract_std::{
    ibc_client::ExecuteMsg as IbcClientMsg,
    ibc_host::HostAction,
    manager::ModuleInstallConfig,
    objects::module::{ModuleInfo, ModuleVersion},
    proxy::ExecuteMsg,
    IBC_CLIENT,
};
use cosmwasm_std::{to_json_binary, wasm_execute, Coin, CosmosMsg, Deps};
use serde::Serialize;

use super::{AbstractApi, ApiIdentification};
use crate::{
    features::{AccountIdentification, ModuleIdentification},
    AbstractSdkResult, ModuleInterface,
};

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
        host_chain: String, // The chain on which you want to create an account
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msgs: vec![abstract_std::ibc_client::ExecuteMsg::Register {
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
                manager_msg: abstract_std::manager::ExecuteMsg::InstallModules {
                    modules: vec![ModuleInstallConfig::new(
                        module,
                        Some(to_json_binary(&init_msg)?),
                    )],
                },
            },
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
                manager_msg: abstract_std::manager::ExecuteMsg::InstallModules {
                    modules: vec![ModuleInstallConfig::new(module, None)],
                },
            },
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
                manager_msg: abstract_std::manager::ExecuteMsg::ExecOnModule {
                    module_id,
                    exec_msg: to_json_binary(exec_msg)?,
                },
            },
        )
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
                msgs: vec![IbcClientMsg::RemoteAction { host_chain, action }],
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
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, *};
    use speculoos::prelude::*;

    use super::*;
    use crate::mock_module::*;
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
                manager_msg: abstract_std::manager::ExecuteMsg::UpdateStatus { is_suspended: None },
            },
        );
        assert_that!(msg).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_json_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::RemoteAction {
                    host_chain: TEST_HOST_CHAIN.into(),
                    action: HostAction::Dispatch {
                        manager_msg: abstract_std::manager::ExecuteMsg::UpdateStatus {
                            is_suspended: None,
                        },
                    },
                }],
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
