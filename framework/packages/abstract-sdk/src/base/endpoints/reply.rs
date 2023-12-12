use crate::base::Handler;
use cosmwasm_std::{Reply, Response};

/// Trait for a contract's Reply entry point.
pub trait ReplyEndpoint: Handler {
    /// Handler for the Reply endpoint.
    fn reply(mut self, msg: Reply) -> Result<Response, Self::Error> {
        let id = msg.id;
        let handler = self.reply_handler(id)?;
        handler(&mut self, msg)?;
        Ok(self._generate_response()?)
    }
}
