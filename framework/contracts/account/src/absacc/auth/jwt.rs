use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use cosmos_sdk_proto::traits::MessageExt;
use cosmos_sdk_proto::xion::v1::jwk::QueryValidateJwtRequest;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Deps};
use serde::{Deserialize, Serialize};
use std::str;

use crate::{contract::AccountResult, error::AccountError};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // aud: Box<[String]>, // Optional. Audience
    // exp: u64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    // iat: u64, // Optional. Issued at (as UTC timestamp)
    // iss: String, // Optional. Issuer
    // nbf: u64, // Optional. Not Before (as UTC timestamp)
    // sub: String, // Optional. Subject (whom token refers to)
    transaction_hash: Binary,
}

#[cw_serde]
struct PrivateClaims {
    key: String,
    value: String,
}
#[cw_serde]
#[allow(non_snake_case)]
struct QueryValidateJWTResponse {
    privateClaims: Vec<PrivateClaims>,
}

pub fn verify(
    deps: Deps,
    tx_hash: &Vec<u8>,
    sig_bytes: &[u8],
    aud: &str,
    sub: &str,
) -> AccountResult<bool> {
    let query = QueryValidateJwtRequest {
        aud: aud.to_string(),
        sub: sub.to_string(),
        sig_bytes: String::from_utf8(sig_bytes.into())?,
        // tx_hash: challenge,
    };

    let query_bz = query.to_bytes()?;
    deps.querier.query_grpc(
        String::from("/xion.jwk.v1.Query/ValidateJWT"),
        Binary::new(query_bz),
    )?;

    // at this point we have validated the JWT. Any custom claims on it's body
    // can follow
    let mut components = sig_bytes.split(|&b| b == b'.');
    components.next().ok_or(AccountError::InvalidToken {})?; // ignore the header, it is not currently used
    let payload_bytes = components.next().ok_or(AccountError::InvalidToken {})?;
    let payload = URL_SAFE_NO_PAD.decode(payload_bytes)?;
    let claims: Claims = cosmwasm_std::from_json(payload.as_slice())?;

    // make sure the provided hash matches the one from the tx
    if tx_hash.eq(&claims.transaction_hash) {
        Ok(true)
    } else {
        Err(AccountError::InvalidSignatureDetail {
            expected: URL_SAFE_NO_PAD.encode(tx_hash),
            received: URL_SAFE_NO_PAD.encode(claims.transaction_hash),
        })
    }
}
