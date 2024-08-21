use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    features::{
        AbstractNameService, AbstractRegistryAccess, AccountExecutor, AccountIdentification,
    },
    AbstractSdkResult,
};
use cosmwasm_std::{Addr, Deps, StdError};

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
    AbstractNameService
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

/// Retrieve identifying information about the calling Account
impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
    AccountIdentification
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        if let Some(target) = &self.target_account {
            Ok(target.proxy.clone())
        } else {
            Err(StdError::generic_err("No target Account specified to execute on.").into())
        }
    }

    fn manager_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        if let Some(target) = &self.target_account {
            Ok(target.manager.clone())
        } else {
            Err(StdError::generic_err("No Account manager specified.").into())
        }
    }

    fn account_base(
        &self,
        _deps: Deps,
    ) -> AbstractSdkResult<abstract_sdk::std::version_control::AccountBase> {
        if let Some(target) = &self.target_account {
            Ok(target.clone())
        } else {
            Err(StdError::generic_err("No Account base specified.").into())
        }
    }
}

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
    AccountExecutor
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
}

/// Get the version control contract
impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
    AbstractRegistryAccess
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<VersionControlContract> {
        Ok(self.state(deps.storage)?.version_control)
    }
}
#[cfg(test)]
mod tests {
    use abstract_sdk::base::ExecuteEndpoint;
    use abstract_std::{
        adapter::{AdapterRequestMsg, ExecuteMsg},
        version_control::AccountBase,
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, DepsMut, Env, MessageInfo, Response};
    use speculoos::prelude::*;

    use super::*;
    use crate::mock::{
        mock_init, mock_init_custom, MockAdapterContract, MockError, MockExecMsg, MOCK_ADAPTER,
        TEST_METADATA,
    };

    pub fn feature_exec_fn(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        module: MockAdapterContract,
        _msg: MockExecMsg,
    ) -> Result<Response, MockError> {
        let mock_api = MockApi::default();
        let expected_proxy = mock_api.addr_make(TEST_PROXY);
        let expected_manager = mock_api.addr_make(TEST_MANAGER);
        let expected_ans = mock_api.addr_make(TEST_ANS_HOST);
        let expected_vc = mock_api.addr_make(TEST_VERSION_CONTROL);
        // assert with test values
        let proxy = module.proxy_address(deps.as_ref())?;
        assert_that!(proxy).is_equal_to(&expected_proxy);
        let manager = module.manager_address(deps.as_ref())?;
        assert_that!(manager).is_equal_to(&expected_manager);
        let account = module.account_base(deps.as_ref())?;
        assert_that!(account).is_equal_to(AccountBase {
            manager: expected_manager,
            proxy: expected_proxy,
        });
        let ans = module.ans_host(deps.as_ref())?;
        assert_that!(ans).is_equal_to(AnsHost::new(expected_ans));
        let regist = module.abstract_registry(deps.as_ref())?;
        assert_that!(regist).is_equal_to(VersionControlContract::new(expected_vc));

        module.target()?;

        Ok(Response::default())
    }

    pub fn featured_adapter() -> MockAdapterContract {
        MockAdapterContract::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
            .with_execute(feature_exec_fn)
    }

    #[test]
    fn custom_exec() {
        let mut deps = mock_dependencies();
        let manager = deps.api.addr_make(TEST_MANAGER);
        deps.querier = mocked_account_querier_builder().build();

        mock_init_custom(deps.as_mut(), featured_adapter()).unwrap();

        let msg = ExecuteMsg::Module(AdapterRequestMsg {
            proxy_address: None,
            request: MockExecMsg {},
        });

        let res =
            featured_adapter().execute(deps.as_mut(), mock_env(), message_info(&manager, &[]), msg);

        assert_that!(res).is_ok();
    }

    #[test]
    fn targets_not_set() {
        let mut deps = mock_dependencies();
        deps.querier = mocked_account_querier_builder().build();

        mock_init(deps.as_mut()).unwrap();

        let res = MOCK_ADAPTER.proxy_address(deps.as_ref());
        assert_that!(res).is_err();

        let res = MOCK_ADAPTER.manager_address(deps.as_ref());
        assert_that!(res).is_err();

        let res = MOCK_ADAPTER.account_base(deps.as_ref());
        assert_that!(res).is_err();
    }
}
