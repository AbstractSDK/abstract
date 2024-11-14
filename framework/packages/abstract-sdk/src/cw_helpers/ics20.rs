use abstract_std::{
    ibc::{ICS20CallbackPayload, ICS20PacketIdentifier, MsgTransferResponse},
    objects::storage_namespaces::ICS20_CALLBACKS,
};
use cosmwasm_std::{from_json, DepsMut, Reply, StdError, Storage};

use crate::AbstractSdkResult;

/// Reply handler for ics20 callbacks
pub fn ics20_callback_reply(storage: &mut dyn Storage, reply: Reply) -> AbstractSdkResult<()> {
    let res = reply.result.into_result().map_err(StdError::generic_err)?;
    println!("{:x?}", res.data);
    // TODO, implement for msg_responses (to have both cases covered)
    let transfer_response =
        MsgTransferResponse::decode(&res.data.expect("Data is set after sending a packet"))
            .map_err(|e| StdError::generic_err(e.to_string()))?;

    let payload: ICS20CallbackPayload = from_json(reply.payload)?;

    // We register the callback for later use
    cw_storage_plus::Map::new(ICS20_CALLBACKS).save(
        storage,
        ICS20PacketIdentifier {
            channel_id: payload.channel_id,
            sequence: transfer_response.sequence,
        },
        &payload.callback,
    )?;

    Ok(())
}
