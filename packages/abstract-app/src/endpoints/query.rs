use crate::{state::AppContract, AppError};
use crate::{Handler, QueryEndpoint};
use abstract_os::app::AppQueryMsg;
use abstract_sdk::os::app::{AppConfigResponse, BaseQueryMsg, QueryMsg};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw_controllers::AdminResponse;

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg: AppQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > QueryEndpoint
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    type QueryMsg = QueryMsg<CustomQueryMsg>;

    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, Error> {
        match msg {
            QueryMsg::Base(msg) => self.base_query(deps, env, msg).map_err(Into::into),
            QueryMsg::App(msg) => self.query_handler()?(deps, env, self, msg),
        }
    }
}
/// Where we dispatch the queries for the AppContract
/// These BaseQueryMsg declarations can be found in `abstract_sdk::os::common_module::app_msg`
impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
    AppContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    pub fn base_query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            BaseQueryMsg::Admin {} => to_binary(&self.admin(deps)?),
        }
    }

    fn dapp_config(&self, deps: Deps) -> StdResult<AppConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        let admin = self.admin.get(deps)?.unwrap();
        Ok(AppConfigResponse {
            proxy_address: state.proxy_address,
            ans_host_address: state.ans_host.address,
            manager_address: admin,
        })
    }

    fn admin(&self, deps: Deps) -> StdResult<AdminResponse> {
        self.admin.query_admin(deps)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_common::*;

    type AppQueryMsg = QueryMsg<MockQueryMsg>;

    fn query_helper(deps: Deps, msg: AppQueryMsg) -> Result<Binary, MockError> {
        MOCK_APP.query(deps, mock_env(), msg)
    }

    mod app_query {
        use super::*;
        use abstract_sdk::AbstractSdkError;

        #[test]
        fn without_handler() {
            let deps = mock_init();
            let msg = AppQueryMsg::App(MockQueryMsg);

            let res = query_helper(deps.as_ref(), msg);

            assert_that!(res)
                .is_err()
                .matches(|e| {
                    matches!(
                        e,
                        MockError::AbstractSdk(AbstractSdkError::MissingHandler { .. })
                    )
                })
                .matches(|e| e.to_string().contains("query"));
        }

        fn mock_query_handler(
            _deps: Deps,
            _env: Env,
            _contract: &MockAppContract,
            msg: MockQueryMsg,
        ) -> Result<Binary, MockError> {
            // simply return the message as binary
            to_binary(&msg).map_err(Into::into)
        }

        #[test]
        fn with_handler() {
            let deps = mock_init();
            let msg = AppQueryMsg::App(MockQueryMsg);

            let with_mocked_query = MOCK_APP.with_query(mock_query_handler);
            let res = with_mocked_query.query(deps.as_ref(), mock_env(), msg);

            let expected = to_binary(&MockQueryMsg).unwrap();
            assert_that!(res).is_ok().is_equal_to(expected);
        }
    }

    mod base_query {
        use super::*;
        use abstract_testing::{TEST_ANS_HOST, TEST_MANAGER, TEST_PROXY};
        use cosmwasm_std::{from_binary, Addr};

        #[test]
        fn config() -> AppTestResult {
            let deps = mock_init();

            let config_query = QueryMsg::Base(BaseQueryMsg::Config {});
            let res = query_helper(deps.as_ref(), config_query)?;

            assert_that!(from_binary(&res).unwrap()).is_equal_to(AppConfigResponse {
                proxy_address: Addr::unchecked(TEST_PROXY),
                ans_host_address: Addr::unchecked(TEST_ANS_HOST),
                manager_address: Addr::unchecked(TEST_MANAGER),
            });

            Ok(())
        }

        #[test]
        fn admin() -> AppTestResult {
            let deps = mock_init();

            let admin_query = QueryMsg::Base(BaseQueryMsg::Admin {});
            let res = query_helper(deps.as_ref(), admin_query)?;

            assert_that!(from_binary(&res).unwrap()).is_equal_to(AdminResponse {
                admin: Some(TEST_MANAGER.to_string()),
            });

            Ok(())
        }
    }
}
