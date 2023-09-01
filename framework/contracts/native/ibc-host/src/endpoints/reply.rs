use crate::{
    account_commands::{self},
    contract::{HostResponse, HostResult},
    HostError,
};
use abstract_core::ibc_host::state::{REGISTRATION_CACHE, TEMP_ACTION_AFTER_CREATION};
use abstract_sdk::core::abstract_ica::{RegisterResponse, StdAck};
use cosmwasm_std::{DepsMut, Env, Reply, Response};
use cw_utils::MsgExecuteContractResponse;

use super::packet::_handle_host_action;

pub const INIT_CALLBACK_ID: u64 = 7890;
pub const INIT_BEFORE_ACTION_REPLY_ID: u64 = 28379;
pub const RESPONSE_REPLY_ID: u64 = 362738;

/// Handle reply after the Account is created, reply with the proxy address of the created account.
pub fn reply_init_callback(deps: DepsMut, _env: Env, _reply: Reply) -> Result<Response, HostError> {
    // we use storage to pass info from the caller to the reply
    let account_id = REGISTRATION_CACHE.load(deps.storage)?;
    REGISTRATION_CACHE.remove(deps.storage);
    // get the account for the callback
    let account = account_commands::get_account(deps.as_ref(), &account_id)?;

    let data = StdAck::success(RegisterResponse {
        /// return the proxy address of the created account, this allows for coin transfers
        account: account.proxy.into_string(),
    });
    Ok(Response::new().set_data(data))
}

/// Handle reply after the Account is created, reply with the proxy address of the created account.
pub fn reply_execute_action(deps: DepsMut, env: Env, _reply: Reply) -> Result<Response, HostError> {
    // we use storage to pass info from the caller to the reply
    let action_cache = TEMP_ACTION_AFTER_CREATION.load(deps.storage)?;
    TEMP_ACTION_AFTER_CREATION.remove(deps.storage);

    // TODO make sure we are passing the data as well
    _handle_host_action(
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
