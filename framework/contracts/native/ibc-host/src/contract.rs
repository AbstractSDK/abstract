use crate::{
    endpoints::{
        self,
        reply::{
            reply_execute_action, reply_forward_response_data, INIT_BEFORE_ACTION_REPLY_ID,
            RESPONSE_REPLY_ID,
        },
    },
    error::HostError,
};
use abstract_core::{ibc_host::ExecuteMsg, IBC_HOST};
use abstract_macros::abstract_response;
use abstract_sdk::core::ibc_host::{InstantiateMsg, QueryMsg};
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, IbcReceiveResponse, MessageInfo, Reply, Response, StdError,
};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[abstract_response(IBC_HOST)]
pub struct HostResponse;

pub type HostResult<T = Response> = Result<T, HostError>;
pub type IbcHostResult = HostResult<IbcReceiveResponse>;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> HostResult {
    endpoints::instantiate(deps, env, info, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> HostResult {
    // will only process base requests as there is no exec handler set.
    endpoints::execute(deps, env, info, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> HostResult<Binary> {
    // will only process base requests as there is no exec handler set.
    endpoints::query(deps, env, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply_msg: Reply) -> HostResult {
    if reply_msg.id == INIT_BEFORE_ACTION_REPLY_ID {
        reply_execute_action(deps, env, reply_msg)
    } else if reply_msg.id == RESPONSE_REPLY_ID {
        reply_forward_response_data(reply_msg)
    } else {
        Err(HostError::Std(StdError::generic_err("Not implemented")))
    }
}
