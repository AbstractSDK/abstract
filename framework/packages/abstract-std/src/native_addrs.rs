use cosmwasm_std::{
    instantiate2_address, Addr, CanonicalAddr, Deps, Env, QuerierWrapper, StdResult,
};

use crate::AbstractResult;

pub use cw_blob::CHECKSUM as BLOB_CHECKSUM;

// Salts for deployments
pub const ANS_HOST_SALT: &[u8] = b"ans";
pub const REGISTRY_SALT: &[u8] = b"reg";
pub const MODULE_FACTORY_SALT: &[u8] = b"mf";

pub fn ans_address(deps: Deps, abstract_code_id: u64) -> AbstractResult<CanonicalAddr> {
    contract_canon_address(deps, abstract_code_id, ANS_HOST_SALT)
}

pub fn registry_address(deps: Deps, abstract_code_id: u64) -> AbstractResult<CanonicalAddr> {
    contract_canon_address(deps, abstract_code_id, REGISTRY_SALT)
}

pub fn module_factory_address(deps: Deps, abstract_code_id: u64) -> AbstractResult<CanonicalAddr> {
    contract_canon_address(deps, abstract_code_id, MODULE_FACTORY_SALT)
}

/// Address of the abstract admin
pub fn creator_address(
    querier: &QuerierWrapper,
    abstract_code_id: u64,
) -> cosmwasm_std::StdResult<Addr> {
    let code_info = querier.query_wasm_code_info(abstract_code_id)?;
    Ok(code_info.creator)
}

fn _abstract_code_id(
    querier: &QuerierWrapper,
    abstract_address: impl Into<String>,
) -> StdResult<u64> {
    querier
        .query_wasm_contract_info(abstract_address)
        .map(|contract_info| contract_info.code_id)
}

#[cfg(target_arch = "wasm32")]
pub fn abstract_code_id(
    querier: &QuerierWrapper,
    abstract_address: impl Into<String>,
) -> StdResult<u64> {
    // For wasm we can safely use static to avoid recurring queries, as we don't expect multi threads
    static mut ABSTRACT_CODE_ID: Option<u64> = None;
    unsafe {
        if ABSTRACT_CODE_ID.is_none() {
            ABSTRACT_CODE_ID = Some(_abstract_code_id(querier, abstract_address)?)
        }
        Ok(ABSTRACT_CODE_ID.clone().unwrap())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn abstract_code_id(
    querier: &QuerierWrapper,
    abstract_address: impl Into<String>,
) -> StdResult<u64> {
    _abstract_code_id(querier, abstract_address)
}

fn _creator_canon_address(deps: Deps, abstract_code_id: u64) -> StdResult<CanonicalAddr> {
    creator_address(&deps.querier, abstract_code_id).and_then(|creator_addr| {
        deps.api
            .addr_canonicalize(creator_addr.as_str())
            .map(Into::into)
    })
}

#[cfg(target_arch = "wasm32")]
pub fn creator_canon_address(deps: Deps, abstract_code_id: u64) -> StdResult<CanonicalAddr> {
    // For wasm we can safely use static to avoid recurring queries, as we don't expect multi threads
    static mut CREATOR_CANON_ADDRESS: Option<CanonicalAddr> = None;
    unsafe {
        if CREATOR_CANON_ADDRESS.is_none() {
            CREATOR_CANON_ADDRESS = Some(_creator_canon_address(deps, abstract_code_id)?)
        }
        Ok(CREATOR_CANON_ADDRESS.clone().unwrap())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn creator_canon_address(deps: Deps, abstract_code_id: u64) -> StdResult<CanonicalAddr> {
    _creator_canon_address(deps, abstract_code_id)
}

pub fn contract_canon_address(
    deps: Deps,
    abstract_code_id: u64,
    salt: &[u8],
) -> AbstractResult<CanonicalAddr> {
    let creator_canon = creator_canon_address(deps, abstract_code_id)?;
    let canon_addr = instantiate2_address(&BLOB_CHECKSUM, &creator_canon, salt)?;
    Ok(canon_addr)
}

/// Hrp from the address of contract
pub fn hrp_from_env(env: &Env) -> &str {
    hrp_from_address(&env.contract.address)
}

/// Hrp from the address
/// Any address validated on this chain can be used
// https://en.bitcoin.it/wiki/BIP_0173#Specification
pub fn hrp_from_address(addr: &Addr) -> &str {
    addr.as_str()
        .split_once("1")
        .expect("Contract address is not bech32")
        .0
}
