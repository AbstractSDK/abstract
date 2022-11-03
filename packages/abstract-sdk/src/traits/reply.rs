use cosmwasm_std::{DepsMut, Env, Reply, Response, StdError};

pub type ReplyHandlerFn<Module, Error> = fn(DepsMut, Env, Module, Reply) -> Result<Response, Error>;

pub trait ReplyEndpoint: Sized {
    type ContractError: From<cosmwasm_std::StdError>;
    /// Takes request, sets destination and executes request handler
    /// This fn is the only way to get an ApiContract instance which ensures the destination address is set correctly.
    fn reply_handler(&self, id: u64) -> Option<ReplyHandlerFn<Self, Self::ContractError>>;
    fn handle_reply(
        self,
        deps: DepsMut,
        env: Env,
        msg: Reply,
    ) -> Result<Response, Self::ContractError> {
        let id = msg.id;
        let maybe_handler = self.reply_handler(id);
        maybe_handler
            .ok_or_else(|| StdError::GenericErr {
                msg: "Invalid reply id".into(),
            })
            .map(|f| f(deps, env, self, msg))?
    }
}
