use crate::ApiError;
use abstract_core::objects::dependency::StaticDependency;
use abstract_sdk::{
    base::{
        AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
        QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn,
    },
    core::version_control::AccountBase,
    feature_objects::AnsHost,
    namespaces::BASE_STATE,
    AbstractSdkError,
};
use cosmwasm_std::{Addr, Empty, StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Debug};

pub const TRADER_NAMESPACE: &str = "traders";

/// The BaseState contains the main addresses needed for sending and verifying messages
/// Every DApp should use the provided **ans_host** contract for token/contract address resolution.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiState {
    /// Used to verify requests
    pub version_control: Addr,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
}

/// The state variables for our ApiContract.
pub struct ApiContract<
    Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError> + 'static,
    CustomInitMsg: 'static = Empty,
    CustomExecMsg: 'static = Empty,
    CustomQueryMsg: 'static = Empty,
    Receive: 'static = Empty,
> {
    pub(crate) contract:
        AbstractContract<Self, Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, Empty, Receive>,
    pub(crate) base_state: Item<'static, ApiState>,
    /// Map ProxyAddr -> WhitelistedTraders
    pub traders: Map<'static, Addr, HashSet<Addr>>,
    /// The Account on which commands are executed. Set each time in the [`abstract_core::api::ExecuteMsg::Base`] handler.
    pub target_account: Option<AccountBase>,
}

/// Constructor
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    pub const fn new(
        name: &'static str,
        version: &'static str,
        metadata: Option<&'static str>,
    ) -> Self {
        Self {
            contract: AbstractContract::new(name, version, metadata),
            base_state: Item::new(BASE_STATE),
            traders: Map::new(TRADER_NAMESPACE),
            target_account: None,
        }
    }

    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [StaticDependency]) -> Self {
        self.contract = self.contract.with_dependencies(dependencies);
        self
    }

    pub const fn with_replies(
        mut self,
        reply_handlers: &'static [(u64, ReplyHandlerFn<Self, Error>)],
    ) -> Self {
        self.contract = self.contract.with_replies([&[], reply_handlers]);
        self
    }

    /// add IBC callback handler to contract
    pub const fn with_ibc_callbacks(
        mut self,
        callbacks: &'static [(&'static str, IbcCallbackHandlerFn<Self, Error>)],
    ) -> Self {
        self.contract = self.contract.with_ibc_callbacks(callbacks);
        self
    }
    pub const fn with_instantiate(
        mut self,
        instantiate_handler: InstantiateHandlerFn<Self, CustomInitMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_instantiate(instantiate_handler);
        self
    }

    pub const fn with_receive(
        mut self,
        receive_handler: ReceiveHandlerFn<Self, ReceiveMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_receive(receive_handler);
        self
    }

    pub const fn with_execute(
        mut self,
        execute_handler: ExecuteHandlerFn<Self, CustomExecMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_execute(execute_handler);
        self
    }

    pub const fn with_query(
        mut self,
        query_handler: QueryHandlerFn<Self, CustomQueryMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_query(query_handler);
        self
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<ApiState> {
        self.base_state.load(store)
    }

    /// Return the address of the proxy for the Account associated with this API.
    /// Set each time in the [`abstract_core::api::ExecuteMsg::Base`] handler.
    pub fn target(&self) -> Result<&Addr, ApiError> {
        Ok(&self
            .target_account
            .as_ref()
            .ok_or_else(|| StdError::generic_err("No target Account specified to execute on."))?
            .proxy)
    }
}

#[cfg(test)]
mod tests {

    use abstract_testing::prelude::{TEST_MODULE_ID, TEST_VERSION};
    use cosmwasm_std::{to_binary, Response};

    use super::*;
    use crate::mock::{ApiMockResult, MockApiContract, TEST_METADATA};

    const DEP: StaticDependency = StaticDependency::new("module_id", &[">0.0.0"]);

    fn get_mock() -> MockApiContract {
        MockApiContract::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
            .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_exec".as_bytes())))
            .with_query(|_, _, _, _| to_binary("mock_query").map_err(Into::into))
            .with_receive(|_, _, _, _, _| Ok(Response::new().set_data("mock_exec".as_bytes())))
            .with_instantiate(|_, _, _, _, _| Ok(Response::new().set_data("mock_init".as_bytes())))
            .with_dependencies(&[DEP])
            .with_ibc_callbacks(&[("c_id", |_, _, _, _, _, _| {
                Ok(Response::new().set_data("mock_callback".as_bytes()))
            })])
            .with_replies(&[(1u64, |_, _, _, _| {
                Ok(Response::new().set_data("mock_reply".as_bytes()))
            })])
    }

    #[test]
    fn set_handlers() -> ApiMockResult {
        get_mock();
        Ok(())
    }

    #[test]
    fn set_and_get_target() -> ApiMockResult {
        let mut mock = get_mock();
        let target = Addr::unchecked("target");
        mock.target_account = Some(AccountBase {
            proxy: target.clone(),
            manager: Addr::unchecked("manager"),
        });
        assert_eq!(mock.target()?, &target);
        Ok(())
    }
}
