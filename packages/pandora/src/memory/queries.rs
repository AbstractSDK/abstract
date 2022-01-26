use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Binary, Deps, QueryRequest, StdResult, WasmQuery};

use crate::denom::is_denom;
use cosmwasm_storage::to_length_prefixed;
use terraswap::asset::AssetInfo;

/// Query asset infos from Memory Module asset addresses map.
pub fn query_assets_from_mem(
    deps: Deps,
    memory_addr: &Addr,
    asset_names: &[String],
) -> StdResult<BTreeMap<String, AssetInfo>> {
    let mut assets: BTreeMap<String, AssetInfo> = BTreeMap::new();

    for asset in asset_names.iter() {
        let result = deps
            .querier
            .query::<String>(&QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: memory_addr.to_string(),
                // query assets map
                key: Binary::from(concat(&to_length_prefixed(b"assets"), asset.as_bytes())),
            }))?;
        assets.insert(asset.clone(), to_asset_info(deps, result)?);
    }
    Ok(assets)
}

/// Query single asset info from mem
pub fn query_asset_from_mem(
    deps: Deps,
    memory_addr: &Addr,
    asset_name: &str,
) -> StdResult<AssetInfo> {
    let result = deps
        .querier
        .query::<String>(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: memory_addr.to_string(),
            // query assets map
            key: Binary::from(concat(
                &to_length_prefixed(b"assets"),
                asset_name.as_bytes(),
            )),
        }))?;
    to_asset_info(deps, result)
}

/// Query contract addresses from Memory Module contract addresses map.
pub fn query_contracts_from_mem(
    deps: Deps,
    memory_addr: &Addr,
    contract_names: &[String],
) -> StdResult<BTreeMap<String, Addr>> {
    let mut contracts: BTreeMap<String, Addr> = BTreeMap::new();

    // Query over
    for contract in contract_names.iter() {
        let result: Addr = deps
            .querier
            .query::<Addr>(&QueryRequest::Wasm(WasmQuery::Raw {
                contract_addr: memory_addr.to_string(),
                key: Binary::from(concat(
                    // Query contracts map
                    &to_length_prefixed(b"contracts"),
                    contract.as_bytes(),
                )),
            }))?;

        contracts.insert(contract.clone(), result);
    }
    Ok(contracts)
}

/// Query single contract address from mem
pub fn query_contract_from_mem(
    deps: Deps,
    memory_addr: &Addr,
    contract_name: &str,
) -> StdResult<Addr> {
    let result = deps
        .querier
        .query::<String>(&QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: memory_addr.to_string(),
            // query assets map
            key: Binary::from(concat(
                &to_length_prefixed(b"contracts"),
                contract_name.as_bytes(),
            )),
        }))?;
    // Addresses are checked when stored.
    Ok(Addr::unchecked(result))
}

/// Returns the asset info for a given string (either denom or contract addr)
#[inline]
pub fn to_asset_info(deps: Deps, address_or_denom: String) -> StdResult<AssetInfo> {
    return if is_denom(address_or_denom.as_str()) {
        Ok(AssetInfo::NativeToken {
            denom: address_or_denom,
        })
    } else {
        deps.api.addr_validate(address_or_denom.as_str())?;
        Ok(AssetInfo::Token {
            contract_addr: address_or_denom,
        })
    };
}

#[inline]
fn concat(namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut k = namespace.to_vec();
    k.extend_from_slice(key);
    k
}
