use crate::{base::Handler, AbstractSdkError, ModuleInterface};
use abstract_core::{ibc::IbcResponseMsg, IBC_CLIENT};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

/// Trait for a contract's IBC callback ExecuteMsg variant.
pub trait IbcCallbackEndpoint: Handler + ModuleInterface {
    /// Handler for the `ExecuteMsg::IbcCallback()` variant.
    fn ibc_callback(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: IbcResponseMsg,
    ) -> Result<Response, Self::Error> {
        // Todo: Change to use version control instead?
        let ibc_client = self.modules().module_address(IBC_CLIENT)?.clone();
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
            |handler| handler(deps, env, info, self, id, callback_msg, result),
        )
    }
}
