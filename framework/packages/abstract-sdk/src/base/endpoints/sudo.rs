use crate::{base::handler::Handler, features::ResponseGenerator, AbstractSdkError};
use cosmwasm_std::{DepsMut, Env, Response};

/// Trait for a contract's Sudo entry point.
pub trait SudoEndpoint: Handler + ResponseGenerator {
    /// Handler for the Sudo endpoint.
    fn sudo(
        mut self,
        mut deps: DepsMut,
        env: Env,
        msg: <Self as Handler>::SudoMsg,
    ) -> Result<Response, Self::Error> {
        let maybe_handler = self.maybe_sudo_handler();
        maybe_handler.map_or_else(
            || {
                Err(Self::Error::from(AbstractSdkError::MissingHandler {
                    endpoint: "sudo".to_string(),
                }))
            },
            |f| {
                f(deps.branch(), env, &mut self, msg)?;
                Ok(self._generate_response(deps.as_ref())?)
            },
        )
    }
}
