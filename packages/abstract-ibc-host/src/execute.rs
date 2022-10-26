use abstract_os::{
    ibc_host::{ExecuteMsg, HostAction, InternalAction, PacketMsg},
    objects::ChannelEntry,
    ICS20,
};

use abstract_sdk::{MemoryOperation, Resolve};
use cosmwasm_std::{
    from_binary, from_slice, DepsMut, Env, IbcPacketReceiveMsg, IbcReceiveResponse, MessageInfo,
    Response,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::HostError,
    host_commands::{
        receive_balances, receive_dispatch, receive_query, receive_register, receive_send_all_back,
        receive_who_am_i,
    },
    state::{Host, ACCOUNTS, CLOSED_CHANNELS},
};

/// The host contract base implementation.
impl<'a, T: Serialize + DeserializeOwned> Host<'a, T> {
    /// Takes ibc request, matches and executes
    /// This fn is the only way to get an Host instance.
    pub fn handle_packet<RequestError: From<cosmwasm_std::StdError> + From<HostError>>(
        self,
        deps: DepsMut,
        env: Env,
        packet: IbcPacketReceiveMsg,
        packet_handler: impl FnOnce(
            DepsMut,
            Env,
            String,
            Host<T>,
            T,
        ) -> Result<IbcReceiveResponse, RequestError>,
    ) -> Result<IbcReceiveResponse, RequestError> {
        let packet = packet.packet;
        // which local channel did this packet come on
        let channel = packet.dest.channel_id;
        let PacketMsg {
            client_chain,
            os_id,
            action,
            ..
        } = from_slice(&packet.data)?;
        match action {
            HostAction::Internal(InternalAction::Register) => {
                receive_register(deps, env, channel, os_id)
            }
            HostAction::Internal(InternalAction::WhoAmI) => {
                let this_chain = self.base_state.load(deps.storage)?.chain;
                receive_who_am_i(this_chain)
            }
            HostAction::Dispatch { msgs, .. } => receive_dispatch(deps, channel, os_id, msgs),
            HostAction::Query { msgs, .. } => receive_query(deps.as_ref(), msgs),
            HostAction::Balances {} => receive_balances(deps, channel, os_id),
            HostAction::SendAllBack { os_proxy_address } => {
                let mem = self.load_memory(deps.storage)?;
                let ics20_channel_entry = ChannelEntry {
                    connected_chain: client_chain,
                    protocol: ICS20.to_string(),
                };
                let ics20_channel_id = ics20_channel_entry.resolve(deps.as_ref(), &mem)?;

                receive_send_all_back(
                    deps,
                    env,
                    os_id,
                    os_proxy_address.ok_or(HostError::MissingProxyAddress {})?,
                    ics20_channel_id,
                    channel,
                )
            }
            HostAction::App { msg } => {
                return packet_handler(deps, env, channel, self, from_binary(&msg)?)
            }
        }
        .map_err(Into::into)
    }
    pub fn execute(
        &mut self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        message: ExecuteMsg,
    ) -> Result<Response, HostError> {
        match message {
            ExecuteMsg::ClearAccount {
                closed_channel,
                os_id,
            } => {
                let closed_channels = CLOSED_CHANNELS.load(deps.storage)?;
                if !closed_channels.contains(&closed_channel) {
                    return Err(HostError::ChannelNotClosed {});
                }
                // call send_all_back here
                // clean up state
                ACCOUNTS.remove(deps.storage, (&closed_channel, os_id));
                todo!();
            }
        }
    }
}
