use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use schemars::JsonSchema;
use serde::Serialize;

use crate::base::Handler;

pub trait InstantiateEndpoint: Handler {
    type InstantiateMsg: Serialize + JsonSchema;

    /// Instantiate the base contract
    fn instantiate(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Self::Error>;
}
