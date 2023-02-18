use crate::base::handler::Handler;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use schemars::JsonSchema;
use serde::Serialize;

pub trait ExecuteEndpoint: Handler {
    type ExecuteMsg: Serialize + JsonSchema;

    /// Handler for the Execute endpoint.
    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Self::Error>;
}
