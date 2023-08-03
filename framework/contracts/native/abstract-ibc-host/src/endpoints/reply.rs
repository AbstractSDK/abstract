use crate::{
    account_commands::{self, get_account, send_all_back},
    contract::HostResponse,
    state::{
        CHAIN_OF_CHANNEL, CLIENT_PROXY, CONFIG, PROCESSING_PACKET, REGISTRATION_CACHE, RESULTS,
    },
    HostError,
};
use abstract_core::objects::AccountId;
use abstract_sdk::{
    base::{Handler, ReplyEndpoint},
    core::{
        abstract_ica::{DispatchResponse, RegisterResponse, StdAck},
        ibc_host::PacketMsg,
    },
    feature_objects::VersionControlContract,
    AccountVerification,
};
use cosmwasm_std::{DepsMut, Empty, Env, Reply, Response};
use cw_utils::parse_reply_instantiate_data;

pub const RECEIVE_DISPATCH_ID: u64 = 1234;
pub const INIT_CALLBACK_ID: u64 = 7890;

fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, HostError> {
    let id = msg.id;

    let (packet, channel) = PROCESSING_PACKET.load(deps.storage)?;
    PROCESSING_PACKET.remove(deps.storage);
    let client_chain = CHAIN_OF_CHANNEL.load(deps.storage, &channel)?;
    let PacketMsg { account_id, .. } = packet;
    let client_proxy_addr = CLIENT_PROXY.load(deps.storage, &account_id)?;
    let account_base = get_account(deps.as_ref(), &account_id)?;
    // send everything back to client
    let send_back_msg = send_all_back(
        deps.as_ref(),
        env,
        account_base,
        client_proxy_addr,
        client_chain,
    )?;

    Ok(HostResponse::action("reply")
        .add_message(send_back_msg)
        .set_data(StdAck::success(&Empty {})))
}

pub fn reply_dispatch_callback(
    deps: DepsMut,
    _env: Env,
    reply: Reply,
) -> Result<Response, HostError> {
    // add the new result to the current tracker
    let mut results = RESULTS.load(deps.storage)?;
    results.push(reply.result.unwrap().data.unwrap_or_default());
    RESULTS.save(deps.storage, &results)?;

    // update result data if this is the last
    let data = StdAck::success(DispatchResponse { results });
    Ok(Response::new().set_data(data))
}

/// Handle reply after the Account is created, reply with the proxy address of the created account.
pub fn reply_init_callback(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, HostError> {
    // we use storage to pass info from the caller to the reply
    let (channel, account_id): (String, AccountId) = REGISTRATION_CACHE.load(deps.storage)?;
    REGISTRATION_CACHE.remove(deps.storage);
    // get the account for the callback
    let account = account_commands::get_account(deps.as_ref(), &account_id)?;

    // parse contract info from data
    let raw_addr = parse_reply_instantiate_data(reply)
        .map_err(HostError::from)?
        .contract_address;
    let contract_addr = deps.api.addr_validate(&raw_addr)?;

    let data = StdAck::success(RegisterResponse {
        /// return the proxy address of the created account, this allows for coin transfers
        account: account.proxy.into_string(),
    });
    Ok(Response::new().set_data(data))
}
