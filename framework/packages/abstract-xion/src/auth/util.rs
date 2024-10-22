use bech32::{ToBase32, Variant};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

pub const CHAIN_BECH_PREFIX: &str = "xion";
use crate::AbstractXionResult;

pub fn sha256(msg: &[u8]) -> Vec<u8> {
    Sha256::digest(msg).to_vec()
}

fn ripemd160(bytes: &[u8]) -> Vec<u8> {
    Ripemd160::digest(bytes).to_vec()
}

pub fn derive_addr(prefix: &str, pubkey_bytes: &[u8]) -> AbstractXionResult<String> {
    let address_bytes = ripemd160(&sha256(pubkey_bytes));
    let address_str = bech32::encode(prefix, address_bytes.to_base32(), Variant::Bech32)?;

    Ok(address_str)
}
