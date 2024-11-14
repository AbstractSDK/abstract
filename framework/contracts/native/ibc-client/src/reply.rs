use abstract_std::{
    ibc::ICS20PacketIdentifier,
    ibc_client::state::{
        AccountCallbackPayload, ICS20_ACCOUNT_CALLBACKS, ICS20_ACCOUNT_CALLBACK_PAYLOAD,
    },
};
use cosmwasm_std::{DepsMut, Reply, Response, StdError};

use crate::{anybuf::ibc::MsgTransferResponse, contract::IbcClientResult};

pub fn save_callback_actions(deps: DepsMut, reply: Reply) -> IbcClientResult {
    let res = reply.result.into_result().map_err(StdError::generic_err)?;
    // TODO: implement for msg_responses (to have both cases covered)
    let transfer_response =
        MsgTransferResponse::decode(&res.data.expect("Data is set after sending a packet"))
            .map_err(|e| StdError::generic_err(e.to_string()))?;

    // Could be payload on cosmwasm_2_0 supported chains
    // let payload: TokenFlowPayload = from_json(reply.payload)?;
    let payload: AccountCallbackPayload = ICS20_ACCOUNT_CALLBACK_PAYLOAD.load(deps.storage)?;

    // We register the callback for later use
    ICS20_ACCOUNT_CALLBACKS.save(
        deps.storage,
        ICS20PacketIdentifier {
            channel_id: payload.channel_id,
            sequence: transfer_response.sequence,
        },
        &(payload.account_address, payload.funds, payload.msgs),
    )?;

    Ok(Response::new())
}
