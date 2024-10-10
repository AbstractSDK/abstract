use cosmwasm_std::{DepsMut, Env, Reply, Response};

use crate::base::Handler;

/// Trait for a contract's Reply entry point.
pub trait ReplyEndpoint: Handler {
    /// Handler for the Reply endpoint.
    fn reply(self, deps: DepsMut, env: Env, msg: Reply) -> Result<Response, Self::Error> {
        let id = msg.id;
        let handler = self.reply_handler(id)?;
        handler(deps, env, &self, msg).map(|r| r.into_cosmwasm_response(self.contract()))
    }
}
