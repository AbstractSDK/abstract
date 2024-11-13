use cosmwasm_std::{DepsMut, Env, Reply, Response};

use crate::{base::Handler, cw_helpers::ics20_callback_reply};

/// Trait for a contract's Reply entry point.
pub trait ReplyEndpoint: Handler {
    /// Handler for the Reply endpoint.
    fn reply(self, deps: DepsMut, env: Env, msg: Reply) -> Result<Response, Self::Error> {
        let id = msg.id;
        // Handle ICS20 callback Reply if present and id matches
        if self.maybe_ics20_callback_handler() == Some(id) {
            ics20_callback_reply(deps.storage, msg.clone())?;
            // User might want to have extra handling under this reply id
            match self.maybe_reply_handler(id) {
                Some(handler) => handler(deps, env, self, msg),
                None => Ok(Response::new()),
            }
        } else {
            let handler = self.reply_handler(id)?;
            handler(deps, env, self, msg)
        }
    }
}
