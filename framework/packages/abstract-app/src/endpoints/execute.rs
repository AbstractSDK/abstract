use abstract_sdk::base::ModuleIbcEndpoint;
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
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _message: BaseExecuteMsg,
    ) -> AppResult {
        unreachable!("App BaseExecuteMsg could not be constructed")
    }
}
