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
            BaseExecuteMsg::UpdateConfig {} => self.update_config(deps, env, info),
        }
    }

    fn update_config(&self, deps: DepsMut, env: Env, info: MessageInfo) -> AppResult {
        // Only the admin should be able to call this
        self.admin.assert_admin(deps.as_ref(), &env, &info.sender)?;

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
    use cosmwasm_std::{testing::*, Addr, Response};
    use cw_controllers::AdminError;

    type AppExecuteMsg = SuperExecuteMsg<MockExecMsg>;

    fn execute_as(
        deps: &mut MockDeps,
        sender: &Addr,
        msg: AppExecuteMsg,
    ) -> Result<Response, MockError> {
        let info = message_info(sender, &[]);
        let env = mock_env_validated(deps.api);
        MOCK_APP_WITH_DEP.execute(deps.as_mut(), env, info, msg)
    }

    mod base {
        use super::*;

        #[test]
        fn only_account() -> AppTestResult {
            let mut deps = mock_init();

            let msg = AppExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {});

            let not_account = deps.api.addr_make("not_admin");
            let res = execute_as(&mut deps, &not_account, msg);
            assert_eq!(
                res,
                Err(MockError::DappError(AppError::Admin(
                    AdminError::NotAdmin {}
                )))
            );
            Ok(())
        }
    }
}
