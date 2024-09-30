use base64::engine::general_purpose::{self};
use base64::Engine;
use cosmos_sdk_proto::prost::Message;
use cosmos_sdk_proto::traits::MessageExt;
use cosmos_sdk_proto::xion::v1::{
    QueryWebAuthNVerifyAuthenticateRequest, QueryWebAuthNVerifyRegisterRequest,
    QueryWebAuthNVerifyRegisterResponse,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Deps};

use crate::contract::AccountResult;

#[cw_serde]
struct QueryRegisterRequest {
    addr: String,
    challenge: String,
    rp: String,
    data: Binary,
}

#[cw_serde]
struct QueryRegisterResponse {
    credential: Binary,
}

#[cw_serde]
struct QueryAuthenticateResponse {}

pub fn register(deps: Deps, addr: Addr, rp: String, data: Binary) -> AccountResult<Binary> {
    let query = QueryWebAuthNVerifyRegisterRequest {
        addr: addr.clone().into(),
        challenge: Binary::from(addr.as_bytes()).to_base64(),
        rp,
        data: data.to_vec(),
    };

    let query_bz = query.to_bytes()?;
    let query_response = deps.querier.query_grpc(
        String::from("/xion.v1.Query/WebAuthNVerifyRegister"),
        Binary::new(query_bz),
    )?;
    let query_response = QueryWebAuthNVerifyRegisterResponse::decode(query_response.as_slice())?;
    Ok(Binary::new(query_response.credential))
}

#[cw_serde]
struct QueryVerifyRequest {
    addr: String,
    challenge: String,
    rp: String,
    credential: Binary,
    data: Binary,
}

pub fn verify(
    deps: Deps,
    addr: Addr,
    rp: String,
    signature: &Binary,
    tx_hash: Vec<u8>,
    credential: &Binary,
) -> AccountResult<bool> {
    let challenge =
        general_purpose::URL_SAFE_NO_PAD.encode(general_purpose::STANDARD.encode(tx_hash));

    let query = QueryWebAuthNVerifyAuthenticateRequest {
        addr: addr.into(),
        challenge,
        rp,
        credential: credential.clone().into(),
        data: signature.clone().into(),
    };

    let query_bz = query.to_bytes()?;
    deps.querier.query_grpc(
        String::from("/xion.v1.Query/WebAuthNVerifyAuthenticate"),
        Binary::new(query_bz),
    )?;

    Ok(true)
}
