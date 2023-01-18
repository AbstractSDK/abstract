use crate::{state::AppContract, AppError, AppResult};
use crate::{ExecuteEndpoint, Handler, IbcCallbackEndpoint};
use abstract_os::app::AppExecuteMsg;
use abstract_sdk::{
    base::ReceiveEndpoint,
    os::app::{BaseExecuteMsg, ExecuteMsg},
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + 'static,
        CustomExecMsg: Serialize + JsonSchema + AppExecuteMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg: Serialize + JsonSchema,
    > ExecuteEndpoint
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;

    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
        // request_handler: impl FnOnce(DepsMut, Env, MessageInfo, Self, T) -> Result<Response, E>,
    ) -> Result<Response, Error> {
        match msg {
            ExecuteMsg::App(request) => self.execute_handler()?(deps, env, info, self, request),
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info, exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.handle_ibc_callback(deps, env, info, msg),
            abstract_os::base::ExecuteMsg::Receive(msg) => {
                self.handle_receive(deps, env, info, msg)
            }
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported App execute message variant").into()),
        }
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
    AppContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn base_execute(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> AppResult {
        match message {
            BaseExecuteMsg::UpdateConfig { ans_host_address } => {
                self.update_config(deps, info, ans_host_address)
            }
        }
    }

    fn update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        ans_host_address: Option<String>,
    ) -> AppResult {
        // self._update_config(deps, info, ans_host_address)?;
        // Only the admin should be able to call this
        self.admin.assert_admin(deps.as_ref(), &info.sender)?;

        let mut state = self.base_state.load(deps.storage)?;

        if let Some(ans_host_address) = ans_host_address {
            state.ans_host.address = deps.api.addr_validate(ans_host_address.as_str())?;
        }

        self.base_state.save(deps.storage, &state)?;

        Ok(Response::default().add_attribute("action", "update_config"))
    }
}
