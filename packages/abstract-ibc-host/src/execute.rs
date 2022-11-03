use abstract_os::ibc_host::{ExecuteMsg, HostAction, InternalAction, PacketMsg};

use cosmwasm_std::{
    from_binary, from_slice, DepsMut, Env, IbcPacketReceiveMsg, IbcReceiveResponse, MessageInfo,
    Response, StdError,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::HostError,
    host_commands::{receive_query, receive_register, receive_who_am_i},
    state::{Host, ACCOUNTS, CLIENT_PROXY, CLOSED_CHANNELS, PROCESSING_PACKET},
};

/// The host contract base implementation.
impl<'a, T: Serialize + DeserializeOwned> Host<'a, T> {
    /// Takes ibc request, matches and executes
    /// This fn is the only way to get an Host instance.
    pub fn handle_packet<RequestError: From<cosmwasm_std::StdError> + From<HostError>>(
        mut self,
        deps: DepsMut,
        env: Env,
        packet: IbcPacketReceiveMsg,
        packet_handler: impl FnOnce(
            DepsMut,
            Env,
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
        // fill the local proxy address
        self.proxy_address = ACCOUNTS.may_load(deps.storage, (&channel, os_id))?;
        match action {
            HostAction::Internal(InternalAction::Register { os_proxy_address }) => {
                receive_register(deps, env, self, channel, os_id, os_proxy_address)
            }
            HostAction::Internal(InternalAction::WhoAmI) => {
                let this_chain = self.base_state.load(deps.storage)?.chain;
                receive_who_am_i(this_chain)
            }
            HostAction::Dispatch { msgs, .. } => self.receive_dispatch(deps, msgs),
            HostAction::Query { msgs, .. } => receive_query(deps.as_ref(), msgs),
            HostAction::Balances {} => self.receive_balances(deps),
            HostAction::SendAllBack {} => {
                // address of the proxy on the client chain
                let client_proxy_address = CLIENT_PROXY.load(deps.storage, (&channel, os_id))?;
                self.receive_send_all_back(deps, env, client_proxy_address, client_chain)
            }
            HostAction::App { msg } => {
                PROCESSING_PACKET.save(deps.storage, &(from_slice(&packet.data)?, channel))?;
                return packet_handler(deps, env, self, from_binary(&msg)?);
            }
        }
        .map_err(Into::into)
    }
    pub fn execute(
        &mut self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: ExecuteMsg,
    ) -> Result<Response, HostError> {
        match message {
            ExecuteMsg::UpdateConfig {
                memory_address,
                cw1_code_id,
                admin,
            } => self.update_config(deps, info, memory_address, cw1_code_id, admin),
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

    fn update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        memory_address: Option<String>,
        cw1_code_id: Option<u64>,
        admin: Option<String>,
    ) -> Result<Response, HostError> {
        let mut state = self.state(deps.storage)?;

        if info.sender != state.admin {
            return Err(StdError::generic_err("Only admin can update config.").into());
        }

        if let Some(memory_address) = memory_address {
            // validate address format
            state.memory.address = deps.api.addr_validate(&memory_address)?;
        }
        if let Some(cw1_code_id) = cw1_code_id {
            // validate address format
            state.cw1_code_id = cw1_code_id;
        }
        if let Some(admin) = admin {
            // validate address format
            state.admin = deps.api.addr_validate(&admin)?;
        }
        Ok(Response::new())
    }
}
