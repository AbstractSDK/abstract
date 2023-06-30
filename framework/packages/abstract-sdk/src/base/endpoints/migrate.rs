use super::super::Handler;
use cosmwasm_std::{DepsMut, Env, Response};
use schemars::JsonSchema;
use serde::Serialize;

/// Trait for a contract's Migrate entry point.
pub trait MigrateEndpoint: Handler {
    /// The message type for the Migrate entry point.
    type MigrateMsg: Serialize + JsonSchema;

    /// Handler for the Migrate endpoint.
    fn migrate(
        self,
        deps: DepsMut,
        env: Env,
        msg: Self::MigrateMsg,
    ) -> Result<Response, Self::Error>;
}
