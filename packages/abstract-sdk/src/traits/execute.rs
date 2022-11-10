use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use schemars::JsonSchema;
use serde::Serialize;

use crate::base::handler::Handler;

pub trait ExecuteEndpoint: Handler {
    type ExecuteMsg: Serialize + JsonSchema;

    /// Entry point for contract execution
    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Self::Error>;
}
