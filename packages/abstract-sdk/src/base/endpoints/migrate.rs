use cosmwasm_std::{DepsMut, Env, Response};
use schemars::JsonSchema;
use serde::Serialize;

use super::super::Handler;

pub type Name = &'static str;
pub type VersionString = &'static str;

pub trait MigrateEndpoint: Handler {
    type MigrateMsg: Serialize + JsonSchema;
    fn migrate(
        self,
        deps: DepsMut,
        env: Env,
        msg: Self::MigrateMsg,
    ) -> Result<Response, Self::Error>;
}
