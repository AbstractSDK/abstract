use abstract_sdk::{base::ModuleIbcEndpoint, features::AbstractResponse};
use abstract_std::app::{AppExecuteMsg, BaseExecuteMsg, ExecuteMsg};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    state::{AppContract, ContractError},
    AppError, AppResult, ExecuteEndpoint, Handler, IbcCallbackEndpoint,
};

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AppError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_std::AbstractError>
            + 'static,
        CustomInitMsg,
        CustomExecMsg: Serialize + JsonSchema + AppExecuteMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > ExecuteEndpoint
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg>;

    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Error> {
        match msg {
            ExecuteMsg::Module(request) => self.execute_handler()?(deps, env, info, self, request),
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info, exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.ibc_callback(deps, env, info, msg),
            ExecuteMsg::ModuleIbc(msg) => self.module_ibc(deps, env, info, msg),
        }
    }
}

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    fn base_execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> AppResult {
        match message {
            BaseExecuteMsg::UpdateConfig {
                ans_host_address,
                version_control_address,
            } => self.update_config(deps, env, info, ans_host_address, version_control_address),
        }
    }

    fn update_config(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        ans_host_address: Option<String>,
        version_control_address: Option<String>,
    ) -> AppResult {
        // self._update_config(deps, info, ans_host_address)?;
        // Only the admin should be able to call this
        self.admin
            .assert_admin(deps.as_ref(), &env.contract.address, &info.sender)?;

        let mut state = self.base_state.load(deps.storage)?;

        if let Some(ans_host_address) = ans_host_address {
            state.ans_host.address = deps.api.addr_validate(ans_host_address.as_str())?;
        }

        if let Some(version_control_address) = version_control_address {
            state.version_control.address =
                deps.api.addr_validate(version_control_address.as_str())?;
        }

        self.base_state.save(deps.storage, &state)?;

        Ok(self.response("update_config"))
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::ExecuteMsg as SuperExecuteMsg;
    use crate::{mock::*, AppError};
    use abstract_sdk::base::ExecuteEndpoint;
    use abstract_std::app::BaseExecuteMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, Addr, DepsMut, Response};
    use cw_controllers::AdminError;
    use speculoos::prelude::*;

    type AppExecuteMsg = SuperExecuteMsg<MockExecMsg>;

    fn execute_as(deps: DepsMut, sender: &Addr, msg: AppExecuteMsg) -> Result<Response, MockError> {
        let info = message_info(&sender, &[]);
        MOCK_APP_WITH_DEP.execute(deps, mock_env(), info, msg)
    }

    mod base {
        use super::*;

        #[test]
        fn only_manager() -> AppTestResult {
            let mut deps = mock_init();

            let msg = AppExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
                ans_host_address: None,
                version_control_address: None,
            });

            let not_manager = deps.api.addr_make("not_admin");
            let res = execute_as(deps.as_mut(), &not_manager, msg);
            assert_that!(res).is_err().matches(|e| {
                matches!(
                    e,
                    MockError::DappError(AppError::Admin(AdminError::NotAdmin {}))
                )
            });
            Ok(())
        }

        #[test]
        fn update_config_should_update_config() -> AppTestResult {
            let mut deps = mock_init();
            let base = test_account_base(deps.api);

            let new_ans_host = deps.api.addr_make("new_ans_host");
            let new_version_control = deps.api.addr_make("new_version_control");
            let update_ans = AppExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
                ans_host_address: Some(new_ans_host.to_string()),
                version_control_address: Some(new_version_control.to_string()),
            });

            let res = execute_as(deps.as_mut(), &base.manager, update_ans);

            assert_that!(res).is_ok().map(|res| {
                assert_that!(res.messages).is_empty();
                res
            });

            let state = MOCK_APP_WITH_DEP.base_state.load(deps.as_ref().storage)?;

            assert_that!(state.ans_host.address).is_equal_to(new_ans_host);
            assert_that!(state.version_control.address).is_equal_to(new_version_control);

            Ok(())
        }

        #[test]
        fn update_config_with_none_host_should_leave_existing_host() -> AppTestResult {
            let mut deps = mock_init();
            let abstr = AbstractMockAddrs::new(deps.api);

            let update_ans = AppExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
                ans_host_address: None,
                version_control_address: None,
            });

            let res = execute_as(deps.as_mut(), &abstr.account.manager, update_ans);

            assert_that!(res).is_ok().map(|res| {
                assert_that!(res.messages).is_empty();
                res
            });

            let state = MOCK_APP_WITH_DEP.base_state.load(deps.as_ref().storage)?;

            assert_that!(state.ans_host.address).is_equal_to(abstr.ans_host);
            assert_that!(state.version_control.address).is_equal_to(abstr.version_control);

            Ok(())
        }
    }
}
