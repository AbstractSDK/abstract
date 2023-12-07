use crate::base::Handler;
use cosmwasm_std::Response;
use schemars::JsonSchema;
use serde::Serialize;

/// Trait for a contract's Instantiate entry point.
pub trait InstantiateEndpoint: Handler {
    /// The message type for the Instantiate entry point.
    type InstantiateMsg: Serialize + JsonSchema;

    /// Handler for the Instantiate endpoint.
    fn instantiate(self, msg: Self::InstantiateMsg) -> Result<Response, Self::Error>;
}
