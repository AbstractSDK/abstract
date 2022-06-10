use cosmwasm_std::{Addr, QuerierWrapper, StdError};

use cosmwasm_std::StdResult;

use super::state::{Core, OS_ADDRESSES};

pub fn verify_os_manager(
    querier: &QuerierWrapper,
    maybe_manager: &Addr,
    version_control_addr: &Addr,
    os_id: u32,
) -> StdResult<Core> {
    let maybe_os = OS_ADDRESSES.query(querier, version_control_addr.clone(), os_id)?;
    match maybe_os {
        None => Err(StdError::generic_err(format!(
            "OS with id {} is not active.",
            os_id
        ))),
        Some(core) => {
            if &core.manager != maybe_manager {
                Err(StdError::generic_err(
                    "Proposed manager is not the manager of this instance.",
                ))
            } else {
                Ok(core)
            }
        }
    }
}

pub fn verify_os_proxy(
    querier: &QuerierWrapper,
    maybe_proxy: &Addr,
    version_control_addr: &Addr,
    os_id: u32,
) -> StdResult<Core> {
    let maybe_os = OS_ADDRESSES.query(querier, version_control_addr.clone(), os_id)?;
    match maybe_os {
        None => Err(StdError::generic_err(format!(
            "OS with id {} is not active.",
            os_id
        ))),
        Some(core) => {
            if &core.proxy != maybe_proxy {
                Err(StdError::generic_err(
                    "Proposed proxy is not the proxy of this instance.",
                ))
            } else {
                Ok(core)
            }
        }
    }
}

// /// Query the module versions of the modules part of the OS
// pub fn try_raw_code_id_query(
//     deps: Deps,
//     version_control_addr: &Addr,
//     k: (String, String),
// ) -> StdResult<u64> {
//     let path = k.joined_key();
//     deps.querier
//         .query::<u64>(&QueryRequest::Wasm(WasmQuery::Raw {
//             contract_addr: version_control_addr.to_string(),
//             // query assets map
//             key: Binary::from(concat(&to_length_prefixed(b"module_code_ids"), &path)),
//         }))
// }

// /// Query the module versions of the modules part of the OS
// pub fn try_raw_os_manager_query(
//     deps: Deps,
//     version_control_addr: &Addr,
//     os_id: u32,
// ) -> StdResult<Addr> {
//     let path = os_id.joined_key();
//     deps.querier
//         .query::<Addr>(&QueryRequest::Wasm(WasmQuery::Raw {
//             contract_addr: version_control_addr.to_string(),
//             // query assets map
//             key: Binary::from(concat(&to_length_prefixed(b"os_addresses"), &path)),
//         }))
// }

// #[inline]
// fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
//     let mut k = namespace.to_vec();
//     k.extend_from_slice(key);
//     k
// }
