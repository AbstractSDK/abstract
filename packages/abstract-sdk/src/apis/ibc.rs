//! # Ibc Client
//! The IbcClient object provides helper function for ibc-related queries or actions.
//!

use crate::{features::AccountIdentification, AbstractSdkResult};
use abstract_core::{
    ibc_client::{CallbackInfo, ExecuteMsg as IbcClientMsg},
    ibc_host::HostAction,
    proxy::ExecuteMsg,
};
use cosmwasm_std::{wasm_execute, Coin, CosmosMsg, Deps};

/// Interact with other chains over IBC.
pub trait IbcInterface: AccountIdentification {
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

impl<T> IbcInterface for T where T: AccountIdentification {}

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
    /// Call a [`HostAction`] on the host of the provided `host_chain`.
    pub fn host_action(
        &self,
        host_chain: String,
        action: HostAction,
        callback: Option<CallbackInfo>,
        retries: u8,
    ) -> AbstractSdkResult<CosmosMsg> {
        Ok(wasm_execute(
            self.base.proxy_address(self.deps)?.to_string(),
            &ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendPacket {
                    host_chain,
                    action,
                    callback_info: callback,
                    retries,
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
        let expected_retries = 0;
        let msg = client.host_action(
            TEST_HOST_CHAIN.to_string(),
            HostAction::Balances {},
            None,
            expected_retries,
        );
        assert_that!(msg).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendPacket {
                    host_chain: TEST_HOST_CHAIN.to_string(),
                    action: HostAction::Balances {},
                    callback_info: None,
                    retries: expected_retries,
                }],
            })
            .unwrap(),
            funds: vec![],
        });
        assert_that!(msg.unwrap()).is_equal_to(expected);
    }

    /// Tests that a host_action can be built with a callback with more retries
    #[test]
    fn test_host_action_with_callback() {
        let deps = mock_dependencies();
        let stub = MockModule::new();
        let client = stub.ibc_client(deps.as_ref());

        let expected_callback = CallbackInfo {
            id: "callback_id".to_string(),
            receiver: "callback_receiver".to_string(),
        };

        let expected_retries = 50;
        let actual = client.host_action(
            TEST_HOST_CHAIN.to_string(),
            HostAction::Balances {},
            Some(expected_callback.clone()),
            expected_retries,
        );

        assert_that!(actual).is_ok();

        let expected = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_PROXY.to_string(),
            msg: to_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendPacket {
                    host_chain: TEST_HOST_CHAIN.to_string(),
                    action: HostAction::Balances {},
                    callback_info: Some(expected_callback),
                    retries: expected_retries,
                }],
            })
            .unwrap(),
            funds: vec![],
        });

        assert_that!(actual.unwrap()).is_equal_to(expected);
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
            msg: to_binary(&ExecuteMsg::IbcAction {
                msgs: vec![IbcClientMsg::SendFunds {
                    host_chain: TEST_HOST_CHAIN.to_string(),
                    funds: expected_funds,
                }],
            })
            .unwrap(),
            // ensure empty
            funds: vec![],
        });
        assert_that!(msg.unwrap()).is_equal_to(expected);
    }
}
