use crate::base::Handler;
use cosmwasm_std::{Binary, Deps, Env, StdError};

pub trait QueryEndpoint: Handler {
    type QueryMsg;

    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, StdError>;
}
