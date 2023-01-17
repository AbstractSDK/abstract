use crate::base::Handler;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};

pub trait ReceiveEndpoint: Handler {
    fn handle_receive(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: <Self as Handler>::ReceiveMsg,
    ) -> Result<Response, <Self as Handler>::Error> {
        let maybe_handler = self.maybe_receive_handler();
        maybe_handler.map_or_else(
            || {
                Err(Self::Error::from(StdError::generic_err(
                    "Receive endpoint handler not set for module.",
                )))
            },
            |f| f(deps, env, info, self, msg),
        )
    }
}
