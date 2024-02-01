use abstract_core::ibc_host::state::TEMP_ACTION_AFTER_CREATION;
use cosmwasm_std::{DepsMut, Env, Reply, Response};
use cw_utils::MsgExecuteContractResponse;

use super::packet::handle_host_action;
use crate::{
    contract::{HostResponse, HostResult},
    HostError,
};

pub const INIT_BEFORE_ACTION_REPLY_ID: u64 = 28379;
pub const RESPONSE_REPLY_ID: u64 = 362738;

/// Handle reply after the Account is created, reply with the proxy address of the created account.
pub fn reply_execute_action(deps: DepsMut, env: Env, _reply: Reply) -> Result<Response, HostError> {
    // we use storage to pass info from the caller to the reply
    let action_cache = TEMP_ACTION_AFTER_CREATION.load(deps.storage)?;
    TEMP_ACTION_AFTER_CREATION.remove(deps.storage);

    // TODO make sure we are passing the data as well
    handle_host_action(
        deps,
        env,
        action_cache.chain_name,
        action_cache.client_proxy_address,
        action_cache.account_id,
        action_cache.action,
    )
}

/// Add the message's data to the response, if any
pub fn reply_forward_response_data(result: Reply) -> HostResult {
    // get the result from the reply
    let resp = cw_utils::parse_reply_execute_data(result);

    // log and add data if needed
    let resp = if let Ok(MsgExecuteContractResponse { data: Some(data) }) = resp {
        HostResponse::new(
            "forward_response_data_reply",
            vec![("response_data", "true")],
        )
        .set_data(data)
    } else {
        HostResponse::new(
            "forward_response_data_reply",
            vec![("response_data", "false")],
        )
    };

    Ok(resp)
}
