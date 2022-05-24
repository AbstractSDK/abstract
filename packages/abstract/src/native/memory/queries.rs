use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Deps, StdError, StdResult};

use cw_asset::AssetInfo;

use super::state::{ASSET_ADDRESSES, CONTRACT_ADDRESSES};

/// Query asset infos from Memory Module asset addresses map.
pub fn query_assets_from_mem(
    deps: Deps,
    memory_addr: &Addr,
    asset_names: &[String],
) -> StdResult<BTreeMap<String, AssetInfo>> {
    let mut assets: BTreeMap<String, AssetInfo> = BTreeMap::new();

    for asset in asset_names.iter() {
        let result = ASSET_ADDRESSES
            .query(&deps.querier, memory_addr.clone(), asset)?
            .ok_or(StdError::GenericErr {
                msg: "asset not found in memory".to_string(),
            })?;
        assets.insert(asset.clone(), result);
    }
    Ok(assets)
}

/// Query single asset info from mem
pub fn query_asset_from_mem(
    deps: Deps,
    memory_addr: &Addr,
    asset_name: &str,
) -> StdResult<AssetInfo> {
    let result = ASSET_ADDRESSES
        .query(&deps.querier, memory_addr.clone(), asset_name)?
        .ok_or(StdError::GenericErr {
            msg: "asset not found in memory".to_string(),
        })?;
    Ok(result)
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
        let result: Addr = CONTRACT_ADDRESSES
            .query(&deps.querier, memory_addr.clone(), contract)?
            .ok_or(StdError::GenericErr {
                msg: "contract not found in memory".to_string(),
            })?;
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
    let result: Addr = CONTRACT_ADDRESSES
        .query(&deps.querier, memory_addr.clone(), contract_name)?
        .ok_or(StdError::GenericErr {
            msg: "contract not found in memory".to_string(),
        })?;
    // Addresses are checked when stored.
    Ok(Addr::unchecked(result))
}
