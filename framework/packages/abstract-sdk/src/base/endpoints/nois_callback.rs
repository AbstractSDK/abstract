use crate::base::nois_handler::NoisHandler;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use schemars::JsonSchema;
use serde::Serialize;

/// Trait for a contract's Nois callback entry point.
pub trait NoisCallbackEndpoint: NoisHandler {
    /// Handler for the Execute endpoint.
    fn nois_callback(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        callback: nois::NoisCallback,
    ) -> Result<Response, Self::Error>;
}
