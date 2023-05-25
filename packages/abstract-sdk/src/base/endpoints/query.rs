use crate::base::Handler;
use cosmwasm_std::{Binary, Deps, Env};

/// Trait for a contract's Query entry point.
pub trait QueryEndpoint: Handler {
    /// The message type for the Query entry point.
    type QueryMsg;

    /// Handler for the Query endpoint.
    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, Self::Error>;
}
