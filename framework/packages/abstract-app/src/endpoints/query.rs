use crate::{
    state::{AppContract, ContractError},
    Handler, QueryEndpoint,
};
use abstract_core::{
    app::{AppConfigResponse, AppQueryMsg, BaseQueryMsg, QueryMsg},
    objects::module_version::{ModuleDataResponse, MODULE},
};
use abstract_sdk::features::DepsAccess;
use cosmwasm_std::{to_json_binary, Binary, StdResult};
use cw_controllers::AdminResponse;

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
        '_,
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

    fn query(&self, msg: Self::QueryMsg) -> Result<Binary, Error> {
        match msg {
            QueryMsg::Base(msg) => self.base_query(msg).map_err(Into::into),
            QueryMsg::Module(msg) => self.query_handler()?(self, msg),
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
        '_,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    pub fn base_query(&self, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::BaseConfig {} => to_json_binary(&self.dapp_config()?),
            BaseQueryMsg::BaseAdmin {} => to_json_binary(&self.admin()?),
            BaseQueryMsg::ModuleData {} => to_json_binary(&self.module_data()?),
        }
    }

    fn dapp_config(&self) -> StdResult<AppConfigResponse> {
        let state = self.base_state.load(self.deps().storage)?;
        let admin = self.admin.get(self.deps())?.unwrap();
        Ok(AppConfigResponse {
            proxy_address: state.proxy_address,
            ans_host_address: state.ans_host.address,
            manager_address: admin,
        })
    }

    fn admin(&self) -> StdResult<AdminResponse> {
        self.admin.query_admin(self.deps())
    }

    fn module_data(&self) -> StdResult<ModuleDataResponse> {
        let module_data = MODULE.load(self.deps().storage)?;
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::*;
    use cosmwasm_std::Deps;
    use speculoos::prelude::*;

    type AppQueryMsg = QueryMsg<MockQueryMsg>;

    fn query_helper(deps: Deps, msg: AppQueryMsg) -> Result<Binary, MockError> {
        basic_mock_app((deps, mock_env()).into()).query(msg)
    }

    mod app_query {
        use super::*;
        use abstract_sdk::AbstractSdkError;

        #[test]
        fn without_handler() {
            let deps = mock_init();
            let msg = AppQueryMsg::Module(MockQueryMsg);

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
            _contract: &MockAppContract,
            msg: MockQueryMsg,
        ) -> Result<Binary, MockError> {
            // simply return the message as binary
            to_json_binary(&msg).map_err(Into::into)
        }

        #[test]
        fn with_handler() {
            let deps = mock_init();
            let msg = AppQueryMsg::Module(MockQueryMsg);

            let with_mocked_query =
                basic_mock_app((deps.as_ref(), mock_env()).into()).with_query(mock_query_handler);
            let res = with_mocked_query.query(msg);

            let expected = to_json_binary(&MockQueryMsg).unwrap();
            assert_that!(res).is_ok().is_equal_to(expected);
        }
    }

    mod base_query {
        use super::*;
        use abstract_testing::prelude::{TEST_ANS_HOST, TEST_MANAGER, TEST_PROXY};
        use cosmwasm_std::{from_json, Addr};

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
