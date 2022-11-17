use abstract_ibc_host::{
    state::{ACCOUNTS, CLIENT_PROXY, PROCESSING_PACKET},
    HostError,
};
use abstract_sdk::os::ibc_host::PacketMsg;

use cosmwasm_std::{DepsMut, Env, MessageInfo, Reply, Response};

use crate::contract::OsmoHost;

pub fn swap_reply(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    mut host: OsmoHost,
    _reply: Reply,
) -> Result<Response, HostError> {
    let (packet, channel) = PROCESSING_PACKET.load(deps.storage)?;
    let PacketMsg {
        client_chain,
        os_id,
        ..
    } = packet;
    let client_proxy_addr = CLIENT_PROXY.load(deps.storage, (&channel, os_id))?;
    let local_proxy_addr = ACCOUNTS.load(deps.storage, (&channel, os_id))?;
    host.proxy_address = Some(local_proxy_addr);
    // send everything back to client
    let _send_back_msg = host.send_all_back(deps.as_ref(), env, client_proxy_addr, client_chain)?;
    // TODO
    Ok(Response::new())
}
