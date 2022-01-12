use cosmwasm_std::{Addr, Binary};

use cosmwasm_storage::to_length_prefixed;

use cosmwasm_std::{Deps, QueryRequest, StdResult, WasmQuery};

// TODO: Play with the key implementation to move SmartQueries to rawquery implementations

/// Query the module versions of the modules part of the OS
pub fn query_code_id(
    deps: Deps,
    version_control_addr: &Addr,
    module_name: String,
    version: String,
) -> StdResult<u64> {
    deps.querier
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

// Query the module versions of the modules part of the OS
// pub fn query_code_id(
//     deps: Deps,
//     version_control_addr: &Addr,
//     module_name: String,
//     version: String,
// ) -> StdResult<u64> {
//     deps.querier
//         .query::<u64>(&QueryRequest::Wasm(WasmQuery::Raw {
//             contract_addr: version_control_addr.to_string(),
//             // query assets map
//             key: Binary::from(
//                 nested_namespaces_with_key(
//                 &[b"module_code_ids"],
//                 &[module_name.as_bytes()],
//                 version.as_bytes(),
//             )),
//         }))
// }

// pub(crate) fn nested_namespaces_with_key(
//     top_names: &[&[u8]],
//     sub_names: &[&[u8]],
//     key: &[u8],
// ) -> Vec<u8> {
//     let mut size = key.len();
//     for &namespace in top_names {
//         size += namespace.len() + 2;
//     }
//     for &namespace in sub_names {
//         size += namespace.len() + 2;
//     }

//     let mut out = Vec::with_capacity(size);
//     for &namespace in top_names {
//         out.extend_from_slice(&encode_length(namespace));
//         out.extend_from_slice(namespace);
//     }
//     for &namespace in sub_names {
//         out.extend_from_slice(&encode_length(namespace));
//         out.extend_from_slice(namespace);
//     }
//     out.extend_from_slice(key);
//     out
// }

// pub(crate) fn encode_length(namespace: &[u8]) -> [u8; 2] {
//     if namespace.len() > 0xFFFF {
//         panic!("only supports namespaces up to length 0xFFFF")
//     }
//     let length_bytes = (namespace.len() as u32).to_be_bytes();
//     [length_bytes[2], length_bytes[3]]
// }
