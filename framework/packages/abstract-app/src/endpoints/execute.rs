use crate::{
    state::{AppContract, ContractError},
    AppError, AppResult, ExecuteEndpoint, Handler, IbcCallbackEndpoint,
};
use abstract_core::app::{AppExecuteMsg, BaseExecuteMsg, ExecuteMsg};
use abstract_sdk::{
    base::ReceiveEndpoint,
    features::{AbstractResponse, DepsAccess, ResponseGenerator},
};
use cosmwasm_std::{Response, StdError};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AppError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_core::AbstractError>
            + 'static,
        CustomInitMsg,
        CustomExecMsg: Serialize + JsonSchema + AppExecuteMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg: Serialize + JsonSchema,
        SudoMsg,
    > ExecuteEndpoint
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
    type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;

    fn execute(mut self, msg: Self::ExecuteMsg) -> Result<Response, Error> {
        match msg {
            ExecuteMsg::Module(request) => {
                self.execute_handler()?(&mut self, request)?;
                Ok(self._generate_response()?)
            }
            ExecuteMsg::Base(exec_msg) => self.base_execute(exec_msg).map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.ibc_callback(msg),
            ExecuteMsg::Receive(msg) => self.receive(msg),
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported App execute message variant").into()),
        }
    }
}

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
    fn base_execute(&mut self, message: BaseExecuteMsg) -> AppResult {
        match message {
            BaseExecuteMsg::UpdateConfig {
                ans_host_address,
                version_control_address,
            } => self.update_config(ans_host_address, version_control_address)?,
        };
        Ok(self._generate_response()?)
    }

    fn update_config(
        &mut self,
        ans_host_address: Option<String>,
        version_control_address: Option<String>,
    ) -> Result<(), AppError> {
        // self._update_config(deps, info, ans_host_address)?;
        // Only the admin should be able to call this
        self.admin
            .assert_admin(self.deps(), &self.message_info().sender)?;

        let mut state = self.base_state.load(self.deps().storage)?;

        if let Some(ans_host_address) = ans_host_address {
            state.ans_host.address = self.api().addr_validate(ans_host_address.as_str())?;
        }

        if let Some(version_control_address) = version_control_address {
            state.version_control.address =
                self.api().addr_validate(version_control_address.as_str())?;
        }

        self.base_state.save(self.deps.deps_mut().storage, &state)?;

        self.tag_response("update_config");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::ExecuteMsg as SuperExecuteMsg;
    use crate::mock::*;
    use crate::AppError;
    use abstract_core::app::BaseExecuteMsg;
    use abstract_sdk::base::ExecuteEndpoint;
    use abstract_testing::prelude::*;
    use cosmwasm_std::Response;
    use cosmwasm_std::{Addr, DepsMut};
    use cw_controllers::AdminError;
    use speculoos::prelude::*;

    type AppExecuteMsg = SuperExecuteMsg<MockExecMsg, MockReceiveMsg>;

    fn execute_as(deps: DepsMut, sender: &str, msg: AppExecuteMsg) -> Result<Response, MockError> {
        let info = mock_info(sender, &[]);
        mock_app((deps, mock_env(), info).into()).execute(msg)
    }

    fn execute_as_manager(deps: DepsMut, msg: AppExecuteMsg) -> Result<Response, MockError> {
        execute_as(deps, TEST_MANAGER, msg)
    }

    fn test_only_manager(_msg: AppExecuteMsg) -> AppTestResult {
        let mut deps = mock_init();
        let msg = AppExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
            ans_host_address: None,
            version_control_address: None,
        });

        let res = execute_as(deps.as_mut(), "not_admin", msg);
        assert_that!(res).is_err().matches(|e| {
            matches!(
                e,
                MockError::DappError(AppError::Admin(AdminError::NotAdmin {}))
            )
        });
        Ok(())
    }

    mod base {
        use super::*;

        #[test]
        fn only_manager() -> AppTestResult {
            let msg = AppExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
                ans_host_address: None,
                version_control_address: None,
            });

            test_only_manager(msg)
        }

        #[test]
        fn update_config_should_update_config() -> AppTestResult {
            let mut deps = mock_init();

            let new_ans_host = "new_ans_host";
            let new_version_control = "new_version_control";
            let update_ans = AppExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
                ans_host_address: Some(new_ans_host.to_string()),
                version_control_address: Some(new_version_control.to_string()),
            });

            let res = execute_as_manager(deps.as_mut(), update_ans);

            assert_that!(res).is_ok().map(|res| {
                assert_that!(res.messages).is_empty();
                res
            });

            let state = mock_app((deps.as_ref(), mock_env()).into())
                .base_state
                .load(deps.as_ref().storage)?;

            assert_that!(state.ans_host.address).is_equal_to(Addr::unchecked(new_ans_host));
            assert_that!(state.version_control.address)
                .is_equal_to(Addr::unchecked(new_version_control));

            Ok(())
        }

        #[test]
        fn update_config_with_none_host_should_leave_existing_host() -> AppTestResult {
            let mut deps = mock_init();

            let update_ans = AppExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
                ans_host_address: None,
                version_control_address: None,
            });

            let res = execute_as_manager(deps.as_mut(), update_ans);

            assert_that!(res).is_ok().map(|res| {
                assert_that!(res.messages).is_empty();
                res
            });

            let state = mock_app((deps.as_ref(), mock_env()).into())
                .base_state
                .load(deps.as_ref().storage)?;

            assert_that!(state.ans_host.address).is_equal_to(Addr::unchecked(TEST_ANS_HOST));
            assert_that!(state.version_control.address)
                .is_equal_to(Addr::unchecked(TEST_VERSION_CONTROL));

            Ok(())
        }
    }
}
