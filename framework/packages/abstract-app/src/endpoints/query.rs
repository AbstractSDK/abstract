use abstract_sdk::feature_objects::{AnsHost, RegistryContract};
use abstract_std::{
    app::{AppConfigResponse, AppQueryMsg, BaseQueryMsg, QueryMsg},
    objects::{
        gov_type::TopLevelOwnerResponse,
        module_version::{ModuleDataResponse, MODULE},
        ownership::nested_admin::query_top_level_owner_addr,
    },
};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError, StdResult};
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
        SudoMsg,
    > QueryEndpoint
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
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
/// These BaseQueryMsg declarations can be found in `abstract_sdk::std::common_module::app_msg`
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    pub fn base_query(&self, deps: Deps, env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::BaseConfig {} => to_json_binary(&self.dapp_config(deps, &env)?),
            BaseQueryMsg::BaseAdmin {} => to_json_binary(&self.admin(deps)?),
            BaseQueryMsg::ModuleData {} => to_json_binary(&self.module_data(deps)?),
            BaseQueryMsg::TopLevelOwner {} => to_json_binary(&self.top_level_owner(deps)?),
        }
    }

    fn dapp_config(&self, deps: Deps, env: &Env) -> StdResult<AppConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        Ok(AppConfigResponse {
            account: state.account.into_addr(),
            ans_host_address: AnsHost::new(deps.api, env)
                .map_err(|e| StdError::generic_err(e.to_string()))?
                .address,
            registry_address: RegistryContract::new(deps.api, env)
                .map_err(|e| StdError::generic_err(e.to_string()))?
                .address,
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
        let account = self.admin.get(deps)?.unwrap();
        let addr = query_top_level_owner_addr(&deps.querier, account)?;
        Ok(TopLevelOwnerResponse { address: addr })
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_sdk::base::QueryEndpoint;
    use cosmwasm_std::{Binary, Deps, Env};

    use super::QueryMsg as SuperQueryMsg;
    use crate::mock::*;

    type AppQueryMsg = SuperQueryMsg<MockQueryMsg>;

    fn query_helper(deps: Deps, env: Env, msg: AppQueryMsg) -> Result<Binary, MockError> {
        BASIC_MOCK_APP.query(deps, env, msg)
    }

    mod app_query {
        use abstract_sdk::AbstractSdkError;
        use abstract_unit_test_utils::mock_env_validated;
        use cosmwasm_std::{to_json_binary, Env};

        use super::*;

        #[coverage_helper::test]
        fn without_handler() {
            let deps = mock_init();
            let msg = AppQueryMsg::Module(MockQueryMsg::GetSomething {});

            let res = query_helper(deps.as_ref(), mock_env_validated(deps.api), msg);

            assert!(matches!(
                res,
                Err(MockError::AbstractSdk(
                    AbstractSdkError::MissingHandler { .. }
                ))
            ));
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

        #[coverage_helper::test]
        fn with_handler() {
            let deps = mock_init();
            let msg = AppQueryMsg::Module(MockQueryMsg::GetSomething {});

            let with_mocked_query = BASIC_MOCK_APP.with_query(mock_query_handler);
            let res = with_mocked_query.query(deps.as_ref(), mock_env_validated(deps.api), msg);

            let expected = to_json_binary(&MockQueryMsg::GetSomething {}).unwrap();
            assert_eq!(res, Ok(expected));
        }
    }

    mod base_query {
        use super::*;

        use abstract_std::{
            app::{AppConfigResponse, BaseQueryMsg},
            objects::module_version::ModuleDataResponse,
        };
        use abstract_unit_test_utils::prelude::*;
        use cw_controllers::AdminResponse;

        #[coverage_helper::test]
        fn config() -> AppTestResult {
            let deps = mock_init();
            let abstr = AbstractMockAddrs::new(deps.api);
            let account = test_account(deps.api);

            let config_query = QueryMsg::Base(BaseQueryMsg::BaseConfig {});
            let res = query_helper(deps.as_ref(), mock_env_validated(deps.api), config_query)?;

            assert_eq!(
                AppConfigResponse {
                    account: account.into_addr(),
                    ans_host_address: abstr.ans_host,
                    registry_address: abstr.registry,
                },
                from_json(res).unwrap()
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn admin() -> AppTestResult {
            let deps = mock_init();
            let account = test_account(deps.api);

            let admin_query = QueryMsg::Base(BaseQueryMsg::BaseAdmin {});
            let res = query_helper(deps.as_ref(), mock_env_validated(deps.api), admin_query)?;

            assert_eq!(
                AdminResponse {
                    admin: Some(account.addr().to_string()),
                },
                from_json(res).unwrap()
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn module_data() -> AppTestResult {
            let deps = mock_init();

            let module_data_query = QueryMsg::Base(BaseQueryMsg::ModuleData {});
            let res = query_helper(
                deps.as_ref(),
                mock_env_validated(deps.api),
                module_data_query,
            )?;

            assert_eq!(
                ModuleDataResponse {
                    module_id: TEST_MODULE_ID.to_string(),
                    version: TEST_VERSION.to_string(),
                    dependencies: vec![],
                    metadata: None
                },
                from_json(res).unwrap()
            );

            Ok(())
        }
    }
}
