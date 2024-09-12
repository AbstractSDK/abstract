use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    features::{
        AbstractNameService, AbstractRegistryAccess, AccountExecutor, AccountIdentification,
    },
    AbstractSdkResult,
};
use cosmwasm_std::{Deps, StdError};

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    AbstractNameService
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

/// Retrieve identifying information about the calling Account
impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    AccountIdentification
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    fn account(&self, _deps: Deps) -> AbstractSdkResult<abstract_std::version_control::Account> {
        if let Some(target) = &self.target_account {
            Ok(target.clone())
        } else {
            Err(StdError::generic_err("No target Account specified to execute on.").into())
        }
    }
}

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg> AccountExecutor
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
}

/// Get the version control contract
impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    AbstractRegistryAccess
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
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
        version_control::Account,
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
        let abstr = AbstractMockAddrs::new(mock_api);
        let expected_account = abstr.account;
        let expected_ans = abstr.ans_host;
        let expected_vc = abstr.version_control;
        // assert with test values
        let proxy = module.proxy_address(deps.as_ref())?;
        assert_that!(proxy).is_equal_to(&expected_proxy);
        let account = module.account(deps.as_ref())?;
        assert_that!(account).is_equal_to(&expected_account);
        let account = module.account_base(deps.as_ref())?;
        assert_that!(account).is_equal_to(Account::new(expected_account));
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
        let base = test_account_base(deps.api);

        deps.querier = AbstractMockQuerierBuilder::new(deps.api)
            .account(&base, TEST_ACCOUNT_ID)
            .build();

        mock_init_custom(&mut deps, featured_adapter()).unwrap();

        let msg = ExecuteMsg::Module(AdapterRequestMsg {
            account_address: None,
            request: MockExecMsg {},
        });

        let res = featured_adapter().execute(
            deps.as_mut(),
            mock_env(),
            message_info(&base.manager, &[]),
            msg,
        );

        assert_that!(res).is_ok();
    }

    #[test]
    fn targets_not_set() {
        let mut deps = mock_dependencies();
        deps.querier = AbstractMockQuerierBuilder::new(deps.api)
            .account(&test_account_base(deps.api), TEST_ACCOUNT_ID)
            .build();

        mock_init(&mut deps).unwrap();

        let res = MOCK_ADAPTER.proxy_address(deps.as_ref());
        assert_that!(res).is_err();

        let res = MOCK_ADAPTER.manager_address(deps.as_ref());
        assert_that!(res).is_err();

        let res = MOCK_ADAPTER.account_base(deps.as_ref());
        assert_that!(res).is_err();
    }
}
