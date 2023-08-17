use crate::{
    account_commands::{self},
    contract::{HostResponse, HostResult},
    state::REGISTRATION_CACHE,
    HostError,
};
use abstract_sdk::core::abstract_ica::{RegisterResponse, StdAck};
use cosmwasm_std::{DepsMut, Env, Reply, Response};

pub const INIT_CALLBACK_ID: u64 = 7890;
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

/// Add the message's data to the response
pub fn reply_forward_response_data(result: Reply) -> HostResult {
    // get the result from the reply
    let resp = cw_utils::parse_reply_execute_data(result)?;

    // log and add data if needed
    let resp = if let Some(data) = resp.data {
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
