use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{base::Handler, AbstractSdkError};

/// Trait for a contract's Untagged ExecuteMsg variant.
pub trait UntaggedEndpoint: Handler {
    /// Handler for the `ExecuteMsg::Untagge()` variant.
    fn untagged(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: <Self as Handler>::UntaggedMsg,
    ) -> Result<Response, <Self as Handler>::Error> {
        let maybe_handler = self.maybe_untagged_handler();
        maybe_handler.map_or_else(
            || {
                Err(Self::Error::from(AbstractSdkError::MissingHandler {
                    endpoint: "untagged".to_string(),
                }))
            },
            |f| f(deps, env, info, self, msg),
        )
    }
}
