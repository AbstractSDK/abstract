use crate::{base::handler::Handler, AbstractSdkError};
use cosmwasm_std::Response;

/// Trait for a contract's Sudo entry point.
pub trait SudoEndpoint: Handler {
    /// Handler for the Sudo endpoint.
    fn sudo(mut self, msg: <Self as Handler>::SudoMsg) -> Result<Response, Self::Error> {
        let maybe_handler = self.maybe_sudo_handler();
        maybe_handler.map_or_else(
            || {
                Err(Self::Error::from(AbstractSdkError::MissingHandler {
                    endpoint: "sudo".to_string(),
                }))
            },
            |f| {
                f(&mut self, msg)?;
                Ok(self._generate_response()?)
            },
        )
    }
}
