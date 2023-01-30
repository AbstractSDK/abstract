use crate::{error::ApiError, state::ApiContract, ApiResult};
use abstract_os::api::ApiExecuteMsg;
use abstract_sdk::{
    base::{
        endpoints::{ExecuteEndpoint, IbcCallbackEndpoint, ReceiveEndpoint},
        Handler,
    },
    os::api::{BaseExecuteMsg, ExecuteMsg},
    Execution, ModuleInterface, Verification,
};
use cosmwasm_std::{
    to_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, WasmMsg,
};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
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
        let sender = &info.sender;
        match msg {
            ExecuteMsg::App(request) => {
                let core = match request.proxy_address {
                    Some(addr) => {
                        let proxy_addr = deps.api.addr_validate(&addr)?;
                        let traders =
                            self.traders.load(deps.storage, proxy_addr).map_err(|_| {
                                ApiError::UnauthorizedTraderApiRequest(info.sender.to_string())
                            })?;
                        if traders.contains(sender) {
                            self.os_register(deps.as_ref())
                                .assert_proxy(&deps.api.addr_validate(&addr)?)?
                        } else {
                            self.os_register(deps.as_ref())
                                .assert_manager(sender)
                                .map_err(|_| {
                                    ApiError::UnauthorizedTraderApiRequest(info.sender.to_string())
                                })?
                        }
                    }
                    None => self
                        .os_register(deps.as_ref())
                        .assert_manager(sender)
                        .map_err(|_| {
                            ApiError::UnauthorizedTraderApiRequest(info.sender.to_string())
                        })?,
                };
                self.target_os = Some(core);
                self.execute_handler()?(deps, env, info, self, request.request)
            }
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info.clone(), exec_msg)
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
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
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

    /// If dependencies are set, remove self from them.
    pub(crate) fn remove_self_from_deps(
        &mut self,
        deps: Deps,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ApiError> {
        let core = self
            .os_register(deps)
            .assert_manager(&info.sender)
            .map_err(|_| ApiError::UnauthorizedApiRequest {})?;
        self.target_os = Some(core);
        let dependencies = self.dependencies();
        let mut msgs: Vec<CosmosMsg> = vec![];
        let applications = self.modules(deps);
        for dep in dependencies {
            let api_addr = applications.module_address(dep.id);
            // just skip if dep is already removed. This means all the traders are already removed.
            if api_addr.is_err() {
                continue;
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api_addr?.into_string(),
                msg: to_binary(&BaseExecuteMsg::UpdateTraders {
                    to_add: vec![],
                    to_remove: vec![env.contract.address.to_string()],
                })?,
                funds: vec![],
            }));
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
    ) -> Result<Response, ApiError> {
        // Either manager or proxy can add/remove traders.
        // This allows other apis to automatically add themselves, allowing for api-cross-calling.
        let core = self
            .os_register(deps.as_ref())
            .assert_manager(&info.sender)?;

        // Manager can only change traders for associated proxy
        let proxy = core.proxy;

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
        Ok(Response::new().add_attribute("action", format!("update_{proxy}_traders")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use abstract_os::{
        api::{BaseInstantiateMsg, InstantiateMsg},
    };
    use abstract_sdk::base::InstantiateEndpoint;
    use abstract_testing::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Empty, StdError,
    };
    
    use speculoos::prelude::*;
    use thiserror::Error;

    type MockApi = ApiContract<MockError, Empty, Empty, Empty, Empty>;
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

    fn mock_exec_handler(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _api: MockApi,
        _msg: Empty,
    ) -> Result<Response, MockError> {
        Ok(Response::new().set_data("mock_response".as_bytes()))
    }

    fn mock_api() -> MockApi {
        MockApi::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
            .with_execute(mock_exec_handler)
    }

    #[test]
    fn add_trader() -> ApiMockResult {
        let env = mock_env();
        let info = mock_info(TEST_MANAGER, &[]);
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();

        mock_init(deps.as_mut())?;

        let mut api = mock_api();
        let msg = BaseExecuteMsg::UpdateTraders {
            to_add: vec![TEST_TRADER.into()],
            to_remove: vec![],
        };
        // consumes api
        api.base_execute(deps.as_mut(), env, info, msg)?;

        let api = mock_api();
        let no_traders_registered = api.traders.is_empty(&deps.storage);
        assert_that!(no_traders_registered).is_false();

        let test_proxy_traders = api
            .traders
            .load(&deps.storage, Addr::unchecked(TEST_PROXY))?;

        assert_that!(test_proxy_traders).has_length(1);
        assert_that!(test_proxy_traders).contains(Addr::unchecked(TEST_TRADER));
        Ok(())
    }
}
