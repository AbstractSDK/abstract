use crate::xion_proto::jwk::QueryValidateJwtRequest;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use cosmwasm_schema::cw_serde;
use cosmwasm_schema::serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary, Deps};
use prost::Message;
use std::str;

use crate::{error::AbstractXionError, AbstractXionResult};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "::cosmwasm_schema::serde")]
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
    tx_hash: &Binary,
    sig_bytes: &[u8],
    aud: &str,
    sub: &str,
) -> AbstractXionResult<bool> {
    let query = QueryValidateJwtRequest {
        aud: aud.to_string(),
        sub: sub.to_string(),
        sig_bytes: String::from_utf8(sig_bytes.into())?,
        // tx_hash: challenge,
    };

    let query_bz = query.encode_to_vec();
    deps.querier.query_grpc(
        String::from("/xion.jwk.v1.Query/ValidateJWT"),
        Binary::new(query_bz),
    )?;

    // at this point we have validated the JWT. Any custom claims on it's body
    // can follow
    let mut components = sig_bytes.split(|&b| b == b'.');
    components
        .next()
        .ok_or(AbstractXionError::InvalidToken {})?; // ignore the header, it is not currently used
    let payload_bytes = components
        .next()
        .ok_or(AbstractXionError::InvalidToken {})?;
    let payload = URL_SAFE_NO_PAD.decode(payload_bytes)?;
    let claims: Claims = cosmwasm_std::from_json(payload.as_slice())?;

    // make sure the provided hash matches the one from the tx
    if tx_hash.eq(&claims.transaction_hash) {
        Ok(true)
    } else {
        Err(AbstractXionError::InvalidSignatureDetail {
            expected: URL_SAFE_NO_PAD.encode(tx_hash),
            received: URL_SAFE_NO_PAD.encode(claims.transaction_hash),
        })
    }
}
