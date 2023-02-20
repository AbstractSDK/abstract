use crate::{
    endpoints::reply::RECEIVE_DISPATCH_ID, host_commands::PACKET_LIFETIME, state::RESULTS, Host,
    HostError,
};
use abstract_sdk::{
    features::AbstractNameService,
    os::{
        abstract_ica::{BalancesResponse, DispatchResponse, SendAllBackResponse, StdAck},
        objects::ChannelEntry,
        ICS20,
    },
};
use cosmwasm_std::{
    wasm_execute, CosmosMsg, Deps, DepsMut, Empty, Env, IbcMsg, IbcReceiveResponse, SubMsg,
};

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError> + From<abstract_sdk::AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Host<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    // processes PacketMsg::Balances variant
    pub fn receive_balances(&self, deps: DepsMut) -> Result<IbcReceiveResponse, HostError> {
        let account = self.proxy_address.as_ref().unwrap();
        let balances = deps.querier.query_all_balances(account)?;
        let response = BalancesResponse {
            account: account.into(),
            balances,
        };
        let acknowledgement = StdAck::success(&response);
        // and we are golden
        Ok(IbcReceiveResponse::new()
            .set_ack(acknowledgement)
            .add_attribute("action", "receive_balances"))
    }

    // processes PacketMsg::Dispatch variant
    pub fn receive_dispatch(
        &self,
        deps: DepsMut,
        msgs: Vec<CosmosMsg>,
    ) -> Result<IbcReceiveResponse, HostError> {
        let reflect_addr = self.proxy_address.as_ref().unwrap();

        // let them know we're fine
        let response = DispatchResponse { results: vec![] };
        let acknowledgement = StdAck::success(&response);
        // create the message to re-dispatch to the reflect contract
        let reflect_msg = cw1_whitelist::msg::ExecuteMsg::Execute { msgs };
        let wasm_msg = wasm_execute(reflect_addr, &reflect_msg, vec![])?;

        // we wrap it in a submessage to properly report results
        let msg = SubMsg::reply_on_success(wasm_msg, RECEIVE_DISPATCH_ID);

        // reset the data field
        RESULTS.save(deps.storage, &vec![])?;

        Ok(IbcReceiveResponse::new()
            .set_ack(acknowledgement)
            .add_submessage(msg)
            .add_attribute("action", "receive_dispatch"))
    }

    /// processes PacketMsg::SendAllBack variant
    pub fn receive_send_all_back(
        &self,
        deps: DepsMut,
        env: Env,
        client_proxy_address: String,
        client_chain: String,
    ) -> Result<IbcReceiveResponse, HostError> {
        // let them know we're fine
        let response = SendAllBackResponse {};
        let acknowledgement = StdAck::success(&response);

        let wasm_msg =
            self.send_all_back(deps.as_ref(), env, client_proxy_address, client_chain)?;
        // reset the data field
        RESULTS.save(deps.storage, &vec![])?;

        Ok(IbcReceiveResponse::new()
            .set_ack(acknowledgement)
            .add_message(wasm_msg)
            .add_attribute("action", "receive_dispatch"))
    }

    /// construct the msg to send all the assets back
    pub fn send_all_back(
        &self,
        deps: Deps,
        env: Env,
        client_proxy_address: String,
        client_chain: String,
    ) -> Result<CosmosMsg, HostError> {
        let ans = self.name_service(deps);
        let ics20_channel_entry = ChannelEntry {
            connected_chain: client_chain,
            protocol: ICS20.to_string(),
        };
        // get the ics20 channel to send funds back to client
        let ics20_channel_id = ans.query(&ics20_channel_entry)?;

        let reflect_addr = self.proxy_address.as_ref().unwrap();
        let coins = deps.querier.query_all_balances(reflect_addr)?;
        let mut msgs: Vec<CosmosMsg> = vec![];
        for coin in coins {
            msgs.push(
                IbcMsg::Transfer {
                    channel_id: ics20_channel_id.clone(),
                    to_address: client_proxy_address.to_string(),
                    amount: coin,
                    timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
                }
                .into(),
            )
        }
        // create the message to re-dispatch to the reflect contract
        let reflect_msg = cw1_whitelist::msg::ExecuteMsg::Execute { msgs };
        let wasm_msg: CosmosMsg<Empty> = wasm_execute(reflect_addr, &reflect_msg, vec![])?.into();
        Ok(wasm_msg)
    }
}
