use abstract_app::sdk::features::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply, StdError, StdResult};
use protobuf::Message;

use crate::{
    contract::{EtfApp, EtfResult},
    response::MsgInstantiateContractResponse,
    state::STATE,
};

pub fn instantiate_reply(deps: DepsMut, _env: Env, etf: EtfApp, reply: Reply) -> EtfResult {
    let data = reply.result.unwrap().data.unwrap();
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(data.as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let share_token_address = res.get_contract_address();

    let api = deps.api;
    STATE.update(deps.storage, |mut meta| -> StdResult<_> {
        meta.share_token_address = api.addr_validate(share_token_address)?;
        Ok(meta)
    })?;

    Ok(etf.custom_response(
        "instantiate_reply",
        vec![("share_token_address", share_token_address)],
    ))
}
