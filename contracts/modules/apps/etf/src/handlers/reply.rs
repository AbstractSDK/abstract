use crate::contract::{EtfApp, EtfResult};
use crate::response::MsgInstantiateContractResponse;
use abstract_sdk::os::etf::state::STATE;
use cosmwasm_std::{DepsMut, Env, Reply, Response, StdError, StdResult};
use protobuf::Message;

pub fn instantiate_reply(deps: DepsMut, _env: Env, _etf: EtfApp, reply: Reply) -> EtfResult {
    let data = reply.result.unwrap().data.unwrap();
    let res: MsgInstantiateContractResponse =
        Message::parse_from_bytes(data.as_slice()).map_err(|_| {
            StdError::parse_err("MsgInstantiateContractResponse", "failed to parse data")
        })?;
    let liquidity_token = res.get_contract_address();

    let api = deps.api;
    STATE.update(deps.storage, |mut meta| -> StdResult<_> {
        meta.liquidity_token_addr = api.addr_validate(liquidity_token)?;
        Ok(meta)
    })?;

    Ok(Response::new().add_attribute("liquidity_token_addr", liquidity_token))
}
