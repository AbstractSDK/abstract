use crate::{base::Handler, features::ResponseGenerator};
use cosmwasm_std::{DepsMut, Env, Reply, Response};

/// Trait for a contract's Reply entry point.
pub trait ReplyEndpoint: Handler + ResponseGenerator {
    /// Handler for the Reply endpoint.
    fn reply(mut self, mut deps: DepsMut, env: Env, msg: Reply) -> Result<Response, Self::Error> {
        let id = msg.id;
        let handler = self.reply_handler(id)?;
        handler(deps.branch(), env, &mut self, msg)?;
        Ok(self._generate_response(deps.as_ref())?)
    }
}
