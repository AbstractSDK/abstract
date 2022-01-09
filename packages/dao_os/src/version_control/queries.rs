use cosmwasm_std::{Addr, Binary};

use cosmwasm_storage::to_length_prefixed;

use cosmwasm_std::{Deps, QueryRequest, StdResult, WasmQuery};

/// Query the module versions of the modules part of the OS
pub fn query_code_id(
    deps: Deps,
    version_control_addr: &Addr,
    module_name: String,
    version: String,
) -> StdResult<u64> {
    deps
        .querier
        .query::<u64>(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: version_control_addr.to_string(),
            // query assets map
            key: Binary::from(concat(
                &to_length_prefixed(b"module_code_ids"),
                module_name.as_bytes(),
                version.as_bytes(),
            )),
        }))
}

// TODO: improve
#[inline]
fn concat(namespace: &[u8], key1: &[u8], key2: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key1);
    k.extend_from_slice(key2);
    k
}
