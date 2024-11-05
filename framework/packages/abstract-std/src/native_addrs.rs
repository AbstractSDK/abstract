use cosmwasm_std::{instantiate2_address, Addr, Binary, CanonicalAddr, Deps, Env};

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
    querier: &cosmwasm_std::QuerierWrapper,
    abstract_code_id: u64,
) -> cosmwasm_std::StdResult<Addr> {
    let code_info = querier.query_wasm_code_info(abstract_code_id)?;
    Ok(code_info.creator)
}

use std::sync::Once;

// TODO: 2 issues we have with this function
pub fn creator_canon_address(deps: Deps, abstract_code_id: u64) -> CanonicalAddr {
    static CREATOR_CANON_EVALUATION: Once = Once::new();
    // 1) CanonicalAddr cannot be constructed in a const context
    static mut CREATOR_CANON_ADDRESS: Binary = Binary::new(vec![]);
    unsafe {
        CREATOR_CANON_EVALUATION.call_once(|| {
            // 2) StdResult don't have clone, so we have to store just canonical address
            CREATOR_CANON_ADDRESS = creator_address(&deps.querier, abstract_code_id)
                .and_then(|creator_addr| {
                    deps.api
                        .addr_canonicalize(creator_addr.as_str())
                        .map(Into::into)
                })
                .unwrap();
        });
        CREATOR_CANON_ADDRESS.clone().into()
    }
}

pub fn contract_canon_address(
    deps: Deps,
    abstract_code_id: u64,
    salt: &[u8],
) -> AbstractResult<CanonicalAddr> {
    let creator_canon = creator_canon_address(deps, abstract_code_id);
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
