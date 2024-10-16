use crate::AbstractXionResult;

use super::util::{self, derive_addr, sha256};
use base64::{engine::general_purpose, Engine as _};
use cosmwasm_std::{Addr, Api};

pub fn verify(
    api: &dyn Api,
    msg_bytes: &[u8],
    sig_bytes: &[u8],
    pubkey: &[u8],
) -> AbstractXionResult<bool> {
    let signer_s = derive_addr(util::CHAIN_BECH_PREFIX, pubkey)?;
    let signer = api.addr_validate(signer_s.as_str())?;

    let envelope_hash = wrap_message(msg_bytes, signer);

    let verification = api.secp256k1_verify(envelope_hash.as_slice(), sig_bytes, pubkey)?;
    Ok(verification)
}

pub fn wrap_message(msg_bytes: &[u8], signer: Addr) -> Vec<u8> {
    let msg_b64 = general_purpose::STANDARD.encode(msg_bytes);
    // format the msg in the style of ADR-036 SignArbitrary
    let envelope = format!("{{\"account_number\":\"0\",\"chain_id\":\"\",\"fee\":{{\"amount\":[],\"gas\":\"0\"}},\"memo\":\"\",\"msgs\":[{{\"type\":\"sign/MsgSignData\",\"value\":{{\"data\":\"{}\",\"signer\":\"{}\"}}}}],\"sequence\":\"0\"}}", msg_b64.as_str(), signer.as_str());

    sha256(envelope.to_string().as_bytes())
}
