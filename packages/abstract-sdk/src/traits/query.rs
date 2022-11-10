use cosmwasm_std::{Binary, Deps, Env, StdError};

use crate::Handler;

pub trait QueryEndpoint: Handler {
    type QueryMsg;

    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, StdError>;
}
