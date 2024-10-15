use abstract_sdk::{
    feature_objects::{AnsHost, RegistryContract},
    features::{AbstractNameService, AbstractRegistryAccess, AccountIdentification},
    AbstractSdkResult,
};
use cosmwasm_std::{Deps, Env, StdError};

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    AbstractNameService
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    fn ans_host(&self, deps: Deps, env: &Env) -> AbstractSdkResult<AnsHost> {
        AnsHost::new(deps.api, env).map_err(Into::into)
    }
}

/// Retrieve identifying information about the calling Account
impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    AccountIdentification
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    fn account(&self, _deps: Deps) -> AbstractSdkResult<abstract_std::registry::Account> {
        if let Some(target) = &self.target_account {
            Ok(target.clone())
        } else {
            Err(StdError::generic_err("No target Account specified to execute on.").into())
        }
    }
}

/// Get the registry contract
impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    AbstractRegistryAccess
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    fn abstract_registry(&self, deps: Deps, env: &Env) -> AbstractSdkResult<RegistryContract> {
        RegistryContract::new(deps.api, env).map_err(Into::into)
    }
}
#[cfg(test)]
mod tests {
    use abstract_sdk::base::ExecuteEndpoint;
    use abstract_std::adapter::{AdapterRequestMsg, ExecuteMsg};
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, DepsMut, MessageInfo, Response};

    use super::*;
    use crate::mock::{
        mock_init, mock_init_custom, MockAdapterContract, MockError, MockExecMsg, MOCK_ADAPTER,
        TEST_METADATA,
    };

    pub fn feature_exec_fn(
        deps: DepsMut,
        env: Env,
        _info: MessageInfo,
        module: MockAdapterContract,
        _msg: MockExecMsg,
    ) -> Result<Response, MockError> {
        let mock_api = MockApi::default();
        let expected_account = test_account(mock_api);
        // assert with test values
        let account = module.account(deps.as_ref())?;
        assert_eq!(account, expected_account);
        let ans = module.ans_host(deps.as_ref(), &env)?;
        assert_eq!(ans, AnsHost::new(deps.api, &env)?);
        let registry = module.abstract_registry(deps.as_ref(), &env)?;
        assert_eq!(registry, RegistryContract::new(deps.api, &env)?);

        module.target()?;

        Ok(Response::default())
    }

    pub fn featured_adapter() -> MockAdapterContract {
        MockAdapterContract::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
            .with_execute(feature_exec_fn)
    }

    #[coverage_helper::test]
    fn custom_exec() {
        let mut deps = mock_dependencies();
        let account = test_account(deps.api);
        let env = mock_env_validated(deps.api);

        deps.querier = MockQuerierBuilder::new(deps.api)
            .account(&account, TEST_ACCOUNT_ID)
            .build();

        mock_init_custom(&mut deps, featured_adapter()).unwrap();

        let msg = ExecuteMsg::Module(AdapterRequestMsg {
            account_address: None,
            request: MockExecMsg {},
        });

        let res =
            featured_adapter().execute(deps.as_mut(), env, message_info(account.addr(), &[]), msg);

        assert!(res.is_ok());
    }

    #[coverage_helper::test]
    fn targets_not_set() {
        let mut deps = mock_dependencies();
        deps.querier = MockQuerierBuilder::new(deps.api)
            .account(&test_account(deps.api), TEST_ACCOUNT_ID)
            .build();

        mock_init(&mut deps).unwrap();

        let res = MOCK_ADAPTER.account(deps.as_ref());
        assert!(res.is_err());
    }
}
