use crate::{
    account_commands::{self, receive_balances, receive_dispatch, receive_send_all_back},
    error::HostError,
    ibc::{receive_query, receive_register, receive_who_am_i},
    state::{CHAIN_OF_CHANNEL, CLIENT_PROXY},
};
use abstract_core::objects::chain_name::ChainName;
use abstract_sdk::core::ibc_host::{HostAction, InternalAction, PacketMsg};
use cosmwasm_std::{
    from_slice, DepsMut, Env, IbcPacketReceiveMsg, IbcReceiveResponse,
};

/// Takes ibc request, matches and executes
/// This fn is the only way to get an Host instance.
pub fn handle_packet(
    deps: DepsMut,
    env: Env,
    packet: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, HostError> {
    let packet: cosmwasm_std::IbcPacket = packet.packet;
    // which local channel did this packet come on
    let channel = packet.dest.channel_id;
    let client_chain = CHAIN_OF_CHANNEL.load(deps.storage, &channel)?;
    let PacketMsg {
        // client_chain,
        mut account_id,
        action,
        ..
    } = from_slice(&packet.data)?;

    // push the client chain to the account trace
    account_id.trace_mut().push_chain(client_chain.clone());

    // get the local account information
    let account = account_commands::get_account(deps.as_ref(), &account_id)?;
    match action {
        HostAction::Internal(InternalAction::Register {
            description,
            link,
            name,
            account_proxy_address,
        }) => receive_register(
            deps,
            env,
            channel,
            account_id,
            account_proxy_address,
            name,
            description,
            link,
        ),
        HostAction::Internal(InternalAction::WhoAmI { client_chain }) => {
            let this_chain = ChainName::new(&env);
            receive_who_am_i(deps, channel, packet.src, client_chain, this_chain)
        }
        HostAction::Dispatch { manager_msg } => receive_dispatch(deps, account, manager_msg),
        HostAction::Query { msgs, .. } => receive_query(deps.as_ref(), msgs),
        HostAction::Balances {} => receive_balances(deps, account),
        HostAction::SendAllBack {} => {
            // address of the proxy on the client chain
            let client_proxy_address = CLIENT_PROXY.load(deps.storage, &account_id)?;
            receive_send_all_back(deps, env, account, client_proxy_address, client_chain)
        }
        HostAction::App { .. } => todo!(),
    }
    .map_err(Into::into)
}
