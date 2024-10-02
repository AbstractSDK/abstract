use bech32::{Bech32, Hrp};
use cosmwasm_std::{instantiate2_address, Api, CanonicalAddr, Env};
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

use crate::AbstractResult;

pub use cw_blob::CHECKSUM as BLOB_CHECKSUM;

// TODO: fill bytes with Public address of creator
// Default local-juno used right now(for testing)
const TEST_ABSTRACT_CREATOR: [u8; 20] = [
    210, 20, 62, 221, 52, 61, 116, 51, 8, 49, 75, 191, 7, 17, 231, 72, 143, 140, 113, 214,
];

// Salts for deployments
pub const ANS_HOST_SALT: &[u8] = b"ans";
pub const VERSION_CONTROL_SALT: &[u8] = b"vc";
pub const MODULE_FACTORY_SALT: &[u8] = b"mf";
pub const IBC_CLIENT_SALT: &[u8] = b"ic";
pub const IBC_HOST_SALT: &[u8] = b"ih";

pub fn ans_address(hrp: &str, api: &dyn Api) -> AbstractResult<CanonicalAddr> {
    contract_canon_address(hrp, ANS_HOST_SALT, api)
}

pub fn version_control_address(hrp: &str, api: &dyn Api) -> AbstractResult<CanonicalAddr> {
    contract_canon_address(hrp, VERSION_CONTROL_SALT, api)
}

pub fn module_factory_address(hrp: &str, api: &dyn Api) -> AbstractResult<CanonicalAddr> {
    contract_canon_address(hrp, MODULE_FACTORY_SALT, api)
}

pub fn ibc_client_address(hrp: &str, api: &dyn Api) -> AbstractResult<CanonicalAddr> {
    contract_canon_address(hrp, IBC_CLIENT_SALT, api)
}

pub fn ibc_host_address(hrp: &str, api: &dyn Api) -> AbstractResult<CanonicalAddr> {
    contract_canon_address(hrp, IBC_HOST_SALT, api)
}

pub fn derive_addr_from_pub_key(hrp: &str, pub_key: &[u8]) -> AbstractResult<String> {
    let hrp: Hrp = Hrp::parse(hrp)?;

    let hash = Sha256::digest(pub_key);
    let rip_hash = Ripemd160::digest(hash);

    let addr = bech32::encode::<Bech32>(hrp, &rip_hash)?;

    Ok(addr)
}

/// Address of the abstract admin
pub fn creator_address(hrp: &str) -> AbstractResult<String> {
    derive_addr_from_pub_key(hrp, &TEST_ABSTRACT_CREATOR)
}

pub fn contract_canon_address(
    hrp: &str,
    salt: &[u8],
    api: &dyn Api,
) -> AbstractResult<CanonicalAddr> {
    let creator_addr = creator_address(hrp)?;
    let creator_canon = api.addr_canonicalize(&creator_addr)?;
    let canon_addr = instantiate2_address(&BLOB_CHECKSUM, &creator_canon, salt)?;
    Ok(canon_addr)
}

/// Hrp from the address of contract
// https://en.bitcoin.it/wiki/BIP_0173#Specification
pub fn hrp_from_env(env: &Env) -> &str {
    env.contract
        .address
        .as_str()
        .split_once("1")
        .expect("Contract address is not bech32")
        .0
}
