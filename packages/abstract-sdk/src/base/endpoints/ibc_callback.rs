use crate::{base::Handler, AbstractSdkError, ModuleInterface};
use abstract_os::{abstract_ica::IbcResponseMsg, IBC_CLIENT};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub trait IbcCallbackEndpoint: Handler + ModuleInterface {
    /// Takes request, sets destination and executes request handler
    /// This fn is the only way to get an ApiContract instance which ensures the destination address is set correctly.
    fn handle_ibc_callback(
        self,
        deps: DepsMut,
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
        let IbcResponseMsg { id, msg: ack } = msg;
        let maybe_handler = self.maybe_ibc_callback_handler(&id);
        maybe_handler.map_or_else(
            || Ok(Response::new()),
            |handler| handler(deps, env, info, self, id, ack),
        )
    }
}
