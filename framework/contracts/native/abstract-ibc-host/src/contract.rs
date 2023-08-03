use crate::{endpoints, error::HostError};
use abstract_core::{ibc_host::ExecuteMsg, IBC_HOST};
use abstract_macros::abstract_response;
use abstract_sdk::{
    base::{ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint, ReplyEndpoint},
    core::{
        abstract_ica::StdAck,
        ibc_host::{InstantiateMsg, MigrateMsg, QueryMsg},
    },
    Execution,
};
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, IbcPacketReceiveMsg, IbcReceiveResponse,
    MessageInfo, Reply, ReplyOn, Response,
};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[abstract_response(IBC_HOST)]
pub struct HostResponse;

pub type HostResult<T = Response> = Result<T, HostError>;
pub type IbcHostResult = Result<IbcReceiveResponse, HostError>;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> HostResult {
    endpoints::instantiate(deps, env, info, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> HostResult {
    // will only process base requests as there is no exec handler set.
    endpoints::execute(deps, env, info, msg)
}
