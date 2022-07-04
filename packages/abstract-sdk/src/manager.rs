use cosmwasm_std::{to_binary, CosmosMsg, Empty, QuerierWrapper, StdResult, WasmMsg};

use abstract_os::manager::{state::OS_ID, ExecuteMsg::UpdateModuleAddresses};

use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Binary};

use cosmwasm_storage::to_length_prefixed;

use cosmwasm_std::{Deps, QueryRequest, WasmQuery};
use cw2::{ContractVersion, CONTRACT};

pub fn query_os_id(querier: &QuerierWrapper, core_contract_addr: &Addr) -> StdResult<u32> {
    OS_ID.query(querier, core_contract_addr.clone())
}

/// Register the module on the manager
/// can only be called by admin of manager
/// Factory on init
pub fn register_module_on_manager(
    manager_address: String,
    module_name: String,
    module_address: String,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: manager_address,
        msg: to_binary(&UpdateModuleAddresses {
            to_add: Some(vec![(module_name, module_address)]),
            to_remove: None,
        })?,
        funds: vec![],
    }))
}
pub fn query_module_version(deps: &Deps, module_addr: Addr) -> StdResult<ContractVersion> {
    let req = QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: module_addr.into(),
        key: CONTRACT.as_slice().into(),
    });
    deps.querier.query::<ContractVersion>(&req)
}

/// Query the module versions of the modules part of the OS
pub fn query_module_versions(
    deps: Deps,
    manager_addr: &Addr,
    module_names: &[String],
) -> StdResult<BTreeMap<String, ContractVersion>> {
    let addresses: BTreeMap<String, Addr> =
        query_module_addresses(deps, manager_addr, module_names)?;
    let mut module_versions: BTreeMap<String, ContractVersion> = BTreeMap::new();
    for (name, address) in addresses.into_iter() {
        let result = query_module_version(&deps, address)?;
        module_versions.insert(name, result);
    }
    Ok(module_versions)
}

/// Query module addresses from manager
pub fn query_module_addresses(
    deps: Deps,
    manager_addr: &Addr,
    module_names: &[String],
) -> StdResult<BTreeMap<String, Addr>> {
    let mut modules: BTreeMap<String, Addr> = BTreeMap::new();

    // Query over
    for module in module_names.iter() {
        let result: StdResult<Addr> =
            deps.querier
                .query::<Addr>(&QueryRequest::Wasm(WasmQuery::Raw {
                    contract_addr: manager_addr.to_string(),
                    key: Binary::from(concat(
                        // Query modules map
                        &to_length_prefixed(b"os_modules"),
                        module.as_bytes(),
                    )),
                }));
        // Add to map if present, skip otherwise. Allows version control to check what modules are present.
        match result {
            Ok(address) => modules.insert(module.clone(), address),
            Err(_) => None,
        };
    }
    Ok(modules)
}

/// Query single module address from manager
pub fn query_module_address(deps: Deps, manager_addr: &Addr, module_name: &str) -> StdResult<Addr> {
    let result = deps
        .querier
        .query::<String>(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: manager_addr.to_string(),
            // query assets map
            key: Binary::from(concat(
                &to_length_prefixed(b"os_modules"),
                module_name.as_bytes(),
            )),
        }))?;
    // Addresses are checked when stored.
    Ok(Addr::unchecked(result))
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}
