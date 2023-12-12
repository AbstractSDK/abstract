use crate::base::handler::Handler;
use cosmwasm_std::Response;
use schemars::JsonSchema;
use serde::Serialize;

/// Trait for a contract's Execute entry point.
pub trait ExecuteEndpoint: Handler {
    /// The message type for the Execute entry point.
    type ExecuteMsg: Serialize + JsonSchema;

    /// Handler for the Execute endpoint.
    fn execute(self, msg: Self::ExecuteMsg) -> Result<Response, Self::Error>;
}
