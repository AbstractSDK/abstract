use crate::{ApiContract, ApiError};
use abstract_sdk::{
    feature_objects::AnsHost,
    features::{AbstractNameService, AbstractRegistryAccess, Identification},
    AbstractSdkError, AbstractSdkResult,
};
use cosmwasm_std::{Addr, Deps, StdError};

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > AbstractNameService
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

/// Retrieve identifying information about the calling OS
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > Identification
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        if let Some(target) = &self.target_os {
            Ok(target.proxy.clone())
        } else {
            Err(StdError::generic_err("No target OS specified to execute on.").into())
        }
    }

    fn manager_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        if let Some(target) = &self.target_os {
            Ok(target.manager.clone())
        } else {
            Err(StdError::generic_err("No OS manager specified.").into())
        }
    }

    fn os_core(&self, _deps: Deps) -> AbstractSdkResult<abstract_sdk::os::version_control::Core> {
        if let Some(target) = &self.target_os {
            Ok(target.clone())
        } else {
            Err(StdError::generic_err("No OS core specified.").into())
        }
    }
}

/// Get the version control contract
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > AbstractRegistryAccess
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.state(deps.storage)?.version_control)
    }
}
#[cfg(test)]
mod tests {
    use abstract_os::{
        api::{ApiRequestMsg, ExecuteMsg},
        version_control::Core,
    };
    use abstract_sdk::base::ExecuteEndpoint;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        DepsMut, Env, MessageInfo, Response,
    };
    use speculoos::prelude::*;

    use crate::mock::{mock_init_custom, MockApiContract, MockError, MockExecMsg, TEST_METADATA};

    use super::*;

    pub fn feature_exec_fn(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        api: MockApiContract,
        _msg: MockExecMsg,
    ) -> Result<Response, MockError> {
        let proxy = api.proxy_address(deps.as_ref())?;
        // assert with test values
        assert_that!(proxy.as_str()).is_equal_to(TEST_PROXY);
        let manager = api.manager_address(deps.as_ref())?;
        assert_that!(manager.as_str()).is_equal_to(TEST_MANAGER);
        let os_core = api.os_core(deps.as_ref())?;
        assert_that!(os_core).is_equal_to(Core {
            manager: Addr::unchecked(TEST_MANAGER),
            proxy: Addr::unchecked(TEST_PROXY),
        });
        let ans = api.ans_host(deps.as_ref())?;
        assert_that!(ans).is_equal_to(AnsHost::new(Addr::unchecked(TEST_ANS_HOST)));
        let regist = api.abstract_registry(deps.as_ref())?;
        assert_that!(regist.as_str()).is_equal_to(TEST_VERSION_CONTROL);

        api.target()?;

        Ok(Response::default())
    }

    pub fn featured_api() -> MockApiContract {
        MockApiContract::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
            .with_execute(feature_exec_fn)
    }

    #[test]
    fn test_features_with_custom_exec() {
        let mut deps = mock_dependencies();
        deps.querier = mocked_os_querier_builder().build();

        mock_init_custom(deps.as_mut(), featured_api()).unwrap();

        let msg = ExecuteMsg::App(ApiRequestMsg {
            proxy_address: None,
            request: MockExecMsg,
        });

        let res =
            featured_api().execute(deps.as_mut(), mock_env(), mock_info(TEST_MANAGER, &[]), msg);

        assert_that!(res).is_ok();
    }
}
