use cosmwasm_std::{Addr, Binary};

use cosmwasm_std::{Deps, QueryRequest, StdResult, WasmQuery};
use cosmwasm_storage::to_length_prefixed;
use cw_storage_plus::{PrimaryKey, U32Key};

/// Query the module versions of the modules part of the OS
pub fn try_raw_code_id_query(
    deps: Deps,
    version_control_addr: &Addr,
    k: (String, String),
) -> StdResult<u64> {
    let path = k.joined_key();
    deps.querier
        .query::<u64>(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: version_control_addr.to_string(),
            // query assets map
            key: Binary::from(concat(&to_length_prefixed(b"module_code_ids"), &path)),
        }))
}

/// Query the module versions of the modules part of the OS
pub fn try_raw_os_manager_query(
    deps: Deps,
    version_control_addr: &Addr,
    os_id: u32,
) -> StdResult<Addr> {
    let path = U32Key::from(os_id).joined_key();
    deps.querier
        .query::<Addr>(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: version_control_addr.to_string(),
            // query assets map
            key: Binary::from(concat(&to_length_prefixed(b"os_addresses"), &path)),
        }))
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}
