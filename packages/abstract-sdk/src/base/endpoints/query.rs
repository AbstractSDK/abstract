use crate::base::Handler;
use cosmwasm_std::{Binary, Deps, Env};

pub trait QueryEndpoint: Handler {
    type QueryMsg;

    fn query(&self, deps: Deps, env: Env, msg: Self::QueryMsg) -> Result<Binary, Self::Error>;
}
