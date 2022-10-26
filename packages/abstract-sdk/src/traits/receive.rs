use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use serde::Serialize;

pub type ReceiveHandlerFn<App, Msg, Error> =
    fn(DepsMut, Env, MessageInfo, App, Msg) -> Result<Response, Error>;

pub trait ReceiveEndpoint: Sized {
    // abstract out into separate trait
    type ContractError: From<cosmwasm_std::StdError>;
    // Update to serde::Value type later
    type ReceiveMsg: Serialize;

    fn receive_handler(
        &self,
    ) -> Option<ReceiveHandlerFn<Self, Self::ReceiveMsg, Self::ContractError>>;
    fn handle_receive(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ReceiveMsg,
    ) -> Result<Response, Self::ContractError> {
        let maybe_handler = self.receive_handler();
        maybe_handler.map_or_else(|| Ok(Response::new()), |f| f(deps, env, info, self, msg))
    }
}
