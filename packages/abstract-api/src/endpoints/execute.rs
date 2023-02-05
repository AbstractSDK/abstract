use crate::{error::ApiError, state::ApiContract, ApiResult};
use abstract_os::{api::ApiRequestMsg, version_control::Core};
use abstract_sdk::base::features::ModuleIdentification;
use abstract_sdk::{
    apis::respond::AbstractResponse,
    base::{
        endpoints::{ExecuteEndpoint, IbcCallbackEndpoint, ReceiveEndpoint},
        Handler,
    },
    os::api::{ApiExecuteMsg, BaseExecuteMsg, ExecuteMsg},
    Execution, ModuleInterface, OsVerification,
};
use cosmwasm_std::{wasm_execute, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<StdError> + From<ApiError>,
        CustomExecMsg: Serialize + JsonSchema + ApiExecuteMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg: Serialize + JsonSchema,
    > ExecuteEndpoint
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;

    fn execute(
        mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Error> {
        match msg {
            ExecuteMsg::App(request) => self.handle_app_msg(deps, env, info, request),
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info, exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.handle_ibc_callback(deps, env, info, msg),
            ExecuteMsg::Receive(msg) => self.handle_receive(deps, env, info, msg),
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported api execute message variant").into()),
        }
    }
}

/// The api-contract base implementation.
impl<
        Error: From<StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn base_execute(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> ApiResult {
        match message {
            BaseExecuteMsg::UpdateTraders { to_add, to_remove } => {
                self.update_traders(deps, info, to_add, to_remove)
            }
            BaseExecuteMsg::Remove {} => self.remove_self_from_deps(deps.as_ref(), env, info),
        }
    }

    /// Handle a custom execution message sent to this app.
    fn handle_app_msg(
        mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        request: ApiRequestMsg<CustomExecMsg>,
    ) -> Result<Response, Error> {
        let sender = &info.sender;
        let unauthorized_sender = |_| ApiError::UnauthorizedTraderApiRequest {
            api: self.module_id().to_string(),
            sender: sender.to_string(),
        };

        let os_registry = self.os_registry(deps.as_ref());

        let core = match request.proxy_address {
            // The sender must either be a trader or manager.
            Some(requested_proxy) => {
                let proxy_address = deps.api.addr_validate(&requested_proxy)?;
                let requested_core = os_registry.assert_proxy(&proxy_address)?;

                // Load the traders for the given proxy address.
                let traders = self
                    .traders
                    .load(deps.storage, proxy_address)
                    .map_err(unauthorized_sender)?;

                if traders.contains(sender) {
                    // If the sender is a trader, return the core.
                    requested_core
                } else {
                    // If the sender is NOT a trader, check that it is a manager of some OS.
                    os_registry
                        .assert_manager(sender)
                        .map_err(unauthorized_sender)?
                }
            }
            None => os_registry
                .assert_manager(sender)
                .map_err(unauthorized_sender)?,
        };
        self.target_os = Some(core);
        self.execute_handler()?(deps, env, info, self, request.request)
    }

    /// If dependencies are set, remove self from them.
    pub(crate) fn remove_self_from_deps(
        &mut self,
        deps: Deps,
        env: Env,
        info: MessageInfo,
    ) -> ApiResult {
        // Only the manager can remove the API as a dependency.
        let core = self
            .os_registry(deps)
            .assert_manager(&info.sender)
            .map_err(|_| ApiError::UnauthorizedApiRequest {
                api: self.module_id().to_string(),
                sender: info.sender.to_string(),
            })?;
        self.target_os = Some(core);

        let dependencies = self.dependencies();
        let mut msgs: Vec<CosmosMsg> = vec![];
        let modules = self.modules(deps);
        for dep in dependencies {
            let api_addr = modules.module_address(dep.id);
            // just skip if dep is already removed. This means all the traders are already removed.
            if api_addr.is_err() {
                continue;
            };
            msgs.push(
                wasm_execute(
                    api_addr?.into_string(),
                    &BaseExecuteMsg::UpdateTraders {
                        to_add: vec![],
                        to_remove: vec![env.contract.address.to_string()],
                    },
                    vec![],
                )?
                .into(),
            );
        }
        self.executor(deps)
            .execute_with_response(msgs, "remove api from dependencies")
            .map_err(Into::into)
    }

    /// Remove traders from the api.
    fn update_traders(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Vec<String>,
        to_remove: Vec<String>,
    ) -> ApiResult {
        let Core {
            // Manager can only change traders for associated proxy
            proxy,
            ..
        } = self
            .os_registry(deps.as_ref())
            .assert_manager(&info.sender)?;

        let mut traders = self
            .traders
            .may_load(deps.storage, proxy.clone())?
            .unwrap_or_default();

        // Handle the addition of traders
        for trader in to_add {
            let trader_addr = deps.api.addr_validate(trader.as_str())?;
            if !traders.insert(trader_addr) {
                return Err(ApiError::TraderAlreadyPresent { trader });
            }
        }

        // Handling the removal of traders
        for trader in to_remove {
            let trader_addr = deps.api.addr_validate(trader.as_str())?;
            if !traders.remove(&trader_addr) {
                return Err(ApiError::TraderNotPresent { trader });
            }
        }

        self.traders.save(deps.storage, proxy.clone(), &traders)?;
        Ok(self.custom_tag_response(Response::new(), "update_traders", vec![("proxy", proxy)]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abstract_os::{
        api,
        api::{BaseInstantiateMsg, InstantiateMsg},
    };
    use abstract_sdk::base::InstantiateEndpoint;
    use abstract_testing::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Empty, StdError, Storage,
    };
    use std::collections::HashSet;

    use speculoos::prelude::*;
    use thiserror::Error;

    #[cosmwasm_schema::cw_serde]
    struct MockApiExecMsg;

    impl api::ApiExecuteMsg for MockApiExecMsg {}

    type MockApi = ApiContract<MockError, MockApiExecMsg, Empty, Empty, Empty>;
    type ApiMockResult = Result<(), MockError>;

    const TEST_METADATA: &str = "test_metadata";
    const TEST_TRADER: &str = "test_trader";

    #[derive(Error, Debug, PartialEq)]
    enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error(transparent)]
        Api(#[from] ApiError),
    }

    fn mock_init(deps: DepsMut) -> Result<Response, MockError> {
        let api = MockApi::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA));
        let info = mock_info(TEST_ADMIN, &[]);
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: TEST_VERSION_CONTROL.into(),
            },
            app: Empty {},
        };
        api.instantiate(deps, mock_env(), info, init_msg)
    }

    fn mock_api() -> MockApi {
        MockApi::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
            .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_response".as_bytes())))
    }

    fn execute_as(
        deps: DepsMut,
        sender: &str,
        msg: ExecuteMsg<MockApiExecMsg>,
    ) -> Result<Response, MockError> {
        mock_api().execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn base_execute_as(
        deps: DepsMut,
        sender: &str,
        msg: BaseExecuteMsg,
    ) -> Result<Response, MockError> {
        execute_as(deps, sender, api::ExecuteMsg::Base(msg))
    }

    mod update_traders {
        use super::*;

        fn load_test_proxy_traders(storage: &dyn Storage) -> HashSet<Addr> {
            mock_api()
                .traders
                .load(storage, Addr::unchecked(TEST_PROXY))
                .unwrap()
        }

        #[test]
        fn add_trader() -> ApiMockResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            mock_init(deps.as_mut())?;

            let _api = mock_api();
            let msg = BaseExecuteMsg::UpdateTraders {
                to_add: vec![TEST_TRADER.into()],
                to_remove: vec![],
            };

            base_execute_as(deps.as_mut(), TEST_MANAGER, msg)?;

            let api = mock_api();
            assert_that!(api.traders.is_empty(&deps.storage)).is_false();

            let test_proxy_traders = load_test_proxy_traders(&deps.storage);

            assert_that!(test_proxy_traders).has_length(1);
            assert_that!(test_proxy_traders).contains(Addr::unchecked(TEST_TRADER));
            Ok(())
        }

        #[test]
        fn remove_trader() -> ApiMockResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            mock_init(deps.as_mut())?;

            let _api = mock_api();
            let msg = BaseExecuteMsg::UpdateTraders {
                to_add: vec![TEST_TRADER.into()],
                to_remove: vec![],
            };

            base_execute_as(deps.as_mut(), TEST_MANAGER, msg)?;

            let traders = load_test_proxy_traders(&deps.storage);
            assert_that!(traders).has_length(1);

            let msg = BaseExecuteMsg::UpdateTraders {
                to_add: vec![],
                to_remove: vec![TEST_TRADER.into()],
            };

            base_execute_as(deps.as_mut(), TEST_MANAGER, msg)?;
            let traders = load_test_proxy_traders(&deps.storage);
            assert_that!(traders).is_empty();
            Ok(())
        }

        #[test]
        fn add_existing_trader() -> ApiMockResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            mock_init(deps.as_mut())?;

            let _api = mock_api();
            let msg = BaseExecuteMsg::UpdateTraders {
                to_add: vec![TEST_TRADER.into()],
                to_remove: vec![],
            };

            base_execute_as(deps.as_mut(), TEST_MANAGER, msg)?;

            let msg = BaseExecuteMsg::UpdateTraders {
                to_add: vec![TEST_TRADER.into()],
                to_remove: vec![],
            };

            let res = base_execute_as(deps.as_mut(), TEST_MANAGER, msg);

            let _test_trader_string = TEST_TRADER.to_string();
            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    MockError::Api(ApiError::TraderAlreadyPresent {
                        trader: _test_trader_string
                    })
                )
            });

            Ok(())
        }

        #[test]
        fn remove_trader_dne() -> ApiMockResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            mock_init(deps.as_mut())?;

            let _api = mock_api();
            let msg = BaseExecuteMsg::UpdateTraders {
                to_add: vec![],
                to_remove: vec![TEST_TRADER.into()],
            };

            let res = base_execute_as(deps.as_mut(), TEST_MANAGER, msg);

            let _test_trader_string = TEST_TRADER.to_string();
            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    MockError::Api(ApiError::TraderNotPresent {
                        trader: _test_trader_string
                    })
                )
            });

            Ok(())
        }
    }

    mod execute_app {
        use super::*;

        use abstract_testing::mock_module::mocked_os_querier_builder;

        /// This sets up the test with the following:
        /// TEST_PROXY has a single trader, TEST_TRADER
        /// TEST_MANAGER and TEST_PROXY are the OS core
        ///
        /// Note that the querier needs to mock the OS core, as the proxy will
        /// query the OS core to get the list of traders.
        fn setup_with_traders(mut deps: DepsMut, traders: Vec<&str>) {
            mock_init(deps.branch()).unwrap();

            let _api = mock_api();
            let msg = BaseExecuteMsg::UpdateTraders {
                to_add: traders.into_iter().map(Into::into).collect(),
                to_remove: vec![],
            };

            base_execute_as(deps, TEST_MANAGER, msg).unwrap();
        }

        #[test]
        fn not_traders_are_unauthorized() {
            let mut deps = mock_dependencies();
            deps.querier = mocked_os_querier_builder().build();

            setup_with_traders(deps.as_mut(), vec![]);

            let msg = ExecuteMsg::App(ApiRequestMsg {
                proxy_address: None,
                request: MockApiExecMsg,
            });

            let not_trader: String = "someoone".into();
            let res = execute_as(deps.as_mut(), &not_trader, msg);

            assert_unauthorized(res, not_trader);
        }

        fn assert_unauthorized(res: Result<Response, MockError>, _trader: String) {
            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    MockError::Api(ApiError::UnauthorizedTraderApiRequest { sender: _trader, .. })
                )
            });
        }

        #[test]
        fn executing_as_os_manager_is_allowed() {
            let mut deps = mock_dependencies();
            deps.querier = mocked_os_querier_builder().build();

            setup_with_traders(deps.as_mut(), vec![]);

            let msg = ExecuteMsg::App(ApiRequestMsg {
                proxy_address: None,
                request: MockApiExecMsg,
            });

            let res = execute_as(deps.as_mut(), TEST_MANAGER, msg);

            assert_that!(res).is_ok();
        }

        #[test]
        fn executing_as_trader_not_allowed_without_proxy() {
            let mut deps = mock_dependencies();
            deps.querier = mocked_os_querier_builder().build();

            setup_with_traders(deps.as_mut(), vec![TEST_TRADER]);

            let msg = ExecuteMsg::App(ApiRequestMsg {
                proxy_address: None,
                request: MockApiExecMsg,
            });

            let res = execute_as(deps.as_mut(), TEST_TRADER, msg);

            assert_unauthorized(res, TEST_TRADER.into());
        }

        #[test]
        fn executing_as_trader_is_allowed_via_proxy() {
            let mut deps = mock_dependencies();
            deps.querier = mocked_os_querier_builder().build();

            setup_with_traders(deps.as_mut(), vec![TEST_TRADER]);

            let msg = ExecuteMsg::App(ApiRequestMsg {
                proxy_address: Some(TEST_PROXY.into()),
                request: MockApiExecMsg,
            });

            let res = execute_as(deps.as_mut(), TEST_TRADER, msg);

            assert_that!(res).is_ok();
        }

        #[test]
        fn executing_as_trader_on_diff_proxy_should_err() {
            let other_proxy = "some_other_proxy";
            let mut deps = mock_dependencies();
            deps.querier = mocked_os_querier_builder()
                .os("some_other_manager", other_proxy, 69420u32)
                .build();

            setup_with_traders(deps.as_mut(), vec![TEST_TRADER]);

            let msg = ExecuteMsg::App(ApiRequestMsg {
                proxy_address: Some(other_proxy.into()),
                request: MockApiExecMsg,
            });

            let res = execute_as(deps.as_mut(), TEST_TRADER, msg);

            assert_unauthorized(res, TEST_TRADER.into());
        }
    }
}
