use crate::{
    error::HostError,
    host_commands::{receive_query, receive_register, receive_who_am_i},
    state::{ContractError, Host, ACCOUNTS, CLIENT_PROXY, CLOSED_CHANNELS, PROCESSING_PACKET},
};
use abstract_sdk::{
    base::{ExecuteEndpoint, Handler},
    core::ibc_host::{BaseExecuteMsg, ExecuteMsg, HostAction, InternalAction, PacketMsg},
    AccountAction, Execution,
};
use cosmwasm_std::{
    from_binary, from_slice, DepsMut, Env, IbcPacketReceiveMsg, IbcReceiveResponse, MessageInfo,
    Response, StdError,
};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

type HostResult = Result<Response, HostError>;

/// The host contract base implementation.
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg: Serialize + DeserializeOwned + JsonSchema,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg: Serialize + JsonSchema,
        SudoMsg,
    > ExecuteEndpoint
    for Host<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;

    fn execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Self::Error> {
        match msg {
            ExecuteMsg::Module(request) => self.execute_handler()?(deps, env, info, self, request),
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info, exec_msg)
                .map_err(From::from),
            _ => Err(StdError::generic_err("Unsupported Host execute message variant").into()),
        }
    }
}

/// The host contract base implementation.
impl<
        Error: ContractError,
        CustomExecMsg: DeserializeOwned,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
    Host<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg, SudoMsg>
{
    /// Takes ibc request, matches and executes
    /// This fn is the only way to get an Host instance.
    pub fn handle_packet<RequestError: ContractError>(
        mut self,
        deps: DepsMut,
        env: Env,
        packet: IbcPacketReceiveMsg,
        packet_handler: impl FnOnce(
            DepsMut,
            Env,
            Self,
            CustomExecMsg,
        ) -> Result<IbcReceiveResponse, RequestError>,
    ) -> Result<IbcReceiveResponse, RequestError> {
        let packet = packet.packet;
        // which local channel did this packet come on
        let channel = packet.dest.channel_id;
        let PacketMsg {
            client_chain,
            account_id,
            action,
            ..
        } = from_slice(&packet.data)?;
        // fill the local proxy address
        self.proxy_address = ACCOUNTS.may_load(deps.storage, (&channel, account_id))?;
        match action {
            HostAction::Internal(InternalAction::Register {
                account_proxy_address,
            }) => receive_register(deps, env, self, channel, account_id, account_proxy_address),
            HostAction::Internal(InternalAction::WhoAmI) => {
                let this_chain = self.base_state.load(deps.storage)?.chain;
                receive_who_am_i(this_chain)
            }
            HostAction::Dispatch { msgs, .. } => self.receive_dispatch(deps, msgs),
            HostAction::Query { msgs, .. } => receive_query(deps.as_ref(), msgs),
            HostAction::Balances {} => self.receive_balances(deps),
            HostAction::SendAllBack {} => {
                // address of the proxy on the client chain
                let client_proxy_address =
                    CLIENT_PROXY.load(deps.storage, (&channel, account_id))?;
                self.receive_send_all_back(deps, env, client_proxy_address, client_chain)
            }
            HostAction::App { msg } => {
                PROCESSING_PACKET.save(deps.storage, &(from_slice(&packet.data)?, channel))?;
                return packet_handler(deps, env, self, from_binary(&msg)?);
            }
        }
        .map_err(Into::into)
    }

    pub fn base_execute(
        mut self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> HostResult {
        match message {
            BaseExecuteMsg::UpdateAdmin { admin } => {
                let new_admin = deps.api.addr_validate(&admin)?;
                self.admin
                    .execute_update_admin(deps, info, Some(new_admin))
                    .map_err(Into::into)
            }
            BaseExecuteMsg::UpdateConfig {
                ans_host_address,
                cw1_code_id,
            } => self.update_config(deps, info, ans_host_address, cw1_code_id),
            BaseExecuteMsg::RecoverAccount {
                closed_channel,
                account_id,
                msgs,
            } => {
                let closed_channels = CLOSED_CHANNELS.load(deps.storage)?;
                if !closed_channels.contains(&closed_channel) {
                    return Err(HostError::ChannelNotClosed {});
                }
                self.admin.assert_admin(deps.as_ref(), &info.sender)?;
                self.proxy_address =
                    ACCOUNTS.may_load(deps.storage, (&closed_channel, account_id))?;
                ACCOUNTS.remove(deps.storage, (&closed_channel, account_id));
                // Execute provided msgs on proxy.
                self.executor(deps.as_ref())
                    .execute_with_response(vec![AccountAction::from_vec(msgs)], "recover_account")
                    .map_err(Into::into)
            }
        }
    }

    fn update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        ans_host_address: Option<String>,
        cw1_code_id: Option<u64>,
    ) -> HostResult {
        let mut state = self.state(deps.storage)?;

        self.admin.assert_admin(deps.as_ref(), &info.sender)?;

        if let Some(ans_host_address) = ans_host_address {
            // validate address format
            state.ans_host.address = deps.api.addr_validate(&ans_host_address)?;
        }
        if let Some(cw1_code_id) = cw1_code_id {
            // validate address format
            state.cw1_code_id = cw1_code_id;
        }
        self.base_state.save(deps.storage, &state)?;
        Ok(Response::new())
    }
}
