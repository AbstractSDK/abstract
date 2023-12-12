use crate::{base::Handler, AbstractSdkError};
use cosmwasm_std::Response;

/// Trait for a contract's Receive ExecuteMsg variant.
pub trait ReceiveEndpoint: Handler {
    /// Handler for the `ExecuteMsg::Receive()` variant.
    fn receive(
        mut self,
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
                f(&mut self, msg)?;
                Ok(self._generate_response()?)
            },
        )
    }
}
