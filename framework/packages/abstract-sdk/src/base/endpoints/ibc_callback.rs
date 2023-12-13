use crate::{base::Handler, features::ResponseGenerator, AbstractSdkError, ModuleInterface};
use abstract_core::{ibc::IbcResponseMsg, IBC_CLIENT};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

/// Trait for a contract's IBC callback ExecuteMsg variant.
pub trait IbcCallbackEndpoint: Handler + ModuleInterface + ResponseGenerator {
    /// Handler for the `ExecuteMsg::IbcCallback()` variant.
    fn ibc_callback(
        mut self,
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: IbcResponseMsg,
    ) -> Result<Response, Self::Error> {
        // Todo: Change to use version control instead?
        let ibc_client = self.modules(deps.as_ref()).module_address(IBC_CLIENT)?;
        if info.sender.ne(&ibc_client) {
            return Err(AbstractSdkError::CallbackNotCalledByIbcClient {
                caller: info.sender,
                client_addr: ibc_client,
                module: self.info().0.to_string(),
            }
            .into());
        };
        let IbcResponseMsg {
            id,
            msg: callback_msg,
            result,
        } = msg;
        let maybe_handler = self.maybe_ibc_callback_handler(&id);
        maybe_handler.map_or_else(
            || Ok(Response::new()),
            |handler| {
                handler(
                    deps.branch(),
                    env,
                    info,
                    &mut self,
                    id,
                    callback_msg,
                    result,
                )?;
                Ok(self._generate_response(deps.as_ref())?)
            },
        )
    }
}
