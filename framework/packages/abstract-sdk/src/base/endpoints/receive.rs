use crate::{base::Handler, features::ResponseGenerator, AbstractSdkError};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

/// Trait for a contract's Receive ExecuteMsg variant.
pub trait ReceiveEndpoint: Handler + ResponseGenerator {
    /// Handler for the `ExecuteMsg::Receive()` variant.
    fn receive(
        mut self,
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: <Self as Handler>::ReceiveMsg,
    ) -> Result<Response, <Self as Handler>::Error> {
        let maybe_handler = self.maybe_receive_handler();
        maybe_handler.map_or_else(
            || {
                Err(Self::Error::from(AbstractSdkError::MissingHandler {
                    endpoint: "receive".to_string(),
                }))
            },
            |f| {
                f(deps.branch(), env, info, &mut self, msg)?;
                Ok(self._generate_response(deps.as_ref())?)
            },
        )
    }
}
