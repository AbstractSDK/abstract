use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::Handler;

pub trait ReceiveEndpoint: Handler {
    fn handle_receive(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: <Self as Handler>::ReceiveMsg,
    ) -> Result<Response, <Self as Handler>::Error> {
        let maybe_handler = self.maybe_receive_handler();
        maybe_handler.map_or_else(|| Ok(Response::new()), |f| f(deps, env, info, self, msg))
    }
}
