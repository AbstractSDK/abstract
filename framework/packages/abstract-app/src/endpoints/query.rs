use abstract_core::{
    app::{AppConfigResponse, AppQueryMsg, BaseQueryMsg, QueryMsg},
    objects::{
        module_version::{ModuleDataResponse, MODULE},
        nested_admin::{query_top_level_owner, TopLevelOwnerResponse},
    },
};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult};
use cw_controllers::AdminResponse;

use crate::{
    state::{AppContract, ContractError},
    Handler, QueryEndpoint,
};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg: AppQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > QueryEndpoint
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    type QueryMsg = QueryMsg<CustomQueryMsg>;

    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, Error> {
        match msg {
            QueryMsg::Base(msg) => self.base_query(deps, env, msg).map_err(Into::into),
            QueryMsg::Module(msg) => self.query_handler()?(deps, env, self, msg),
        }
    }
}
/// Where we dispatch the queries for the AppContract
/// These BaseQueryMsg declarations can be found in `abstract_sdk::core::common_module::app_msg`
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
    AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    pub fn base_query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::BaseConfig {} => to_json_binary(&self.dapp_config(deps)?),
            BaseQueryMsg::BaseAdmin {} => to_json_binary(&self.admin(deps)?),
            BaseQueryMsg::ModuleData {} => to_json_binary(&self.module_data(deps)?),
            BaseQueryMsg::TopLevelOwner {} => to_json_binary(&self.top_level_owner(deps)?),
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

    fn module_data(&self, deps: Deps) -> StdResult<ModuleDataResponse> {
        let module_data = MODULE.load(deps.storage)?;
        Ok(ModuleDataResponse {
            module_id: module_data.module,
            version: module_data.version,
            dependencies: module_data
                .dependencies
                .into_iter()
                .map(Into::into)
                .collect(),
            metadata: module_data.metadata,
        })
    }

    fn top_level_owner(&self, deps: Deps) -> StdResult<TopLevelOwnerResponse> {
        let manager = self.admin.get(deps)?.unwrap();
        let addr = query_top_level_owner(&deps.querier, manager)?;
        Ok(TopLevelOwnerResponse { address: addr })
    }
}

#[cfg(test)]
mod test {
    use abstract_sdk::base::QueryEndpoint;
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{Binary, Deps};
    use speculoos::prelude::*;

    use super::QueryMsg as SuperQueryMsg;
    use crate::mock::*;

    type AppQueryMsg = SuperQueryMsg<MockQueryMsg>;

    fn query_helper(deps: Deps, msg: AppQueryMsg) -> Result<Binary, MockError> {
        BASIC_MOCK_APP.query(deps, mock_env(), msg)
    }

    mod app_query {
        use abstract_sdk::AbstractSdkError;
        use cosmwasm_std::{to_json_binary, Env};

        use super::*;

        #[test]
        fn without_handler() {
            let deps = mock_init();
            let msg = AppQueryMsg::Module(MockQueryMsg::GetSomething {});

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
            to_json_binary(&msg).map_err(Into::into)
        }

        #[test]
        fn with_handler() {
            let deps = mock_init();
            let msg = AppQueryMsg::Module(MockQueryMsg::GetSomething {});

            let with_mocked_query = BASIC_MOCK_APP.with_query(mock_query_handler);
            let res = with_mocked_query.query(deps.as_ref(), mock_env(), msg);

            let expected = to_json_binary(&MockQueryMsg::GetSomething {}).unwrap();
            assert_that!(res).is_ok().is_equal_to(expected);
        }
    }

    mod base_query {
        use super::*;

        use abstract_core::app::{AppConfigResponse, BaseQueryMsg};
        use abstract_testing::prelude::*;
        use cosmwasm_std::Addr;
        use cw_controllers::AdminResponse;

        #[test]
        fn config() -> AppTestResult {
            let deps = mock_init();

            let config_query = QueryMsg::Base(BaseQueryMsg::BaseConfig {});
            let res = query_helper(deps.as_ref(), config_query)?;

            assert_that!(from_json(res).unwrap()).is_equal_to(AppConfigResponse {
                proxy_address: Addr::unchecked(TEST_PROXY),
                ans_host_address: Addr::unchecked(TEST_ANS_HOST),
                manager_address: Addr::unchecked(TEST_MANAGER),
            });

            Ok(())
        }

        #[test]
        fn admin() -> AppTestResult {
            let deps = mock_init();

            let admin_query = QueryMsg::Base(BaseQueryMsg::BaseAdmin {});
            let res = query_helper(deps.as_ref(), admin_query)?;

            assert_that!(from_json(res).unwrap()).is_equal_to(AdminResponse {
                admin: Some(TEST_MANAGER.to_string()),
            });

            Ok(())
        }
    }
}
