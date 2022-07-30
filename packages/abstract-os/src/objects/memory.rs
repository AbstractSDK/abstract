use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Deps, StdError, StdResult};

use cw_asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::memory::state::{ASSET_ADDRESSES, CONTRACT_ADDRESSES};

use super::memory_entry::ContractEntry;

/// Struct that provides easy in-contract memory querying.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Memory {
    /// Address of the memory contract
    pub address: Addr,
}

impl Memory {
    /// Raw Query to Memory contract
    pub fn query_contracts(
        &self,
        deps: Deps,
        contract_names: &[ContractEntry],
    ) -> StdResult<BTreeMap<ContractEntry, Addr>> {
        query_contracts_from_mem(deps, &self.address, contract_names)
    }

    /// Raw query of a single contract Addr
    pub fn query_contract(&self, deps: Deps, key: &ContractEntry) -> StdResult<Addr> {
        query_contract_from_mem(deps, &self.address, key)
    }

    /// Raw Query to Memory contract
    pub fn query_assets(
        &self,
        deps: Deps,
        asset_names: &[String],
    ) -> StdResult<BTreeMap<String, AssetInfo>> {
        query_assets_from_mem(deps, &self.address, asset_names)
    }

    /// Raw query of a single AssetInfo
    pub fn query_asset(&self, deps: Deps, asset_name: &str) -> StdResult<AssetInfo> {
        query_asset_from_mem(deps, &self.address, asset_name)
    }

    // Query single pair address from mem
    // pub fn query_pair_address(
    //     &self,
    //     deps: Deps,
    //     asset_names: [String; 2],
    //     dex: &str,
    // ) -> StdResult<Addr> {
    //     let mut lowercase = asset_names.map(|s| s.to_ascii_lowercase());
    //     lowercase.sort();
    //     let key = format!("{}_{}", lowercase[0], lowercase[1]);
    //     query_contract_from_mem(deps, &self.address, &ContractEntry::new(dex, &key))
    // }
}

/// Query asset infos from Memory Module asset addresses map.
#[inline(always)]
fn query_assets_from_mem(
    deps: Deps,
    memory_addr: &Addr,
    asset_names: &[String],
) -> StdResult<BTreeMap<String, AssetInfo>> {
    let mut assets: BTreeMap<String, AssetInfo> = BTreeMap::new();

    for asset in asset_names.iter() {
        let result = ASSET_ADDRESSES
            .query(&deps.querier, memory_addr.clone(), asset)?
            .ok_or_else(|| {
                StdError::generic_err(format!("asset {} not found in memory", &asset))
            })?;
        assets.insert(asset.clone(), result);
    }
    Ok(assets)
}

/// Query single asset info from mem
#[inline(always)]
fn query_asset_from_mem(deps: Deps, memory_addr: &Addr, asset_name: &str) -> StdResult<AssetInfo> {
    let result = ASSET_ADDRESSES
        .query(&deps.querier, memory_addr.clone(), asset_name)?
        .ok_or_else(|| {
            StdError::generic_err(format!("asset {} not found in memory", &asset_name))
        })?;
    Ok(result)
}

/// Query contract addresses from Memory Module contract addresses map.
#[inline(always)]
fn query_contracts_from_mem(
    deps: Deps,
    memory_addr: &Addr,
    keys: &[ContractEntry],
) -> StdResult<BTreeMap<ContractEntry, Addr>> {
    let mut contracts: BTreeMap<ContractEntry, Addr> = BTreeMap::new();

    // Query over keys
    for key in keys.iter() {
        let result: Addr = CONTRACT_ADDRESSES
            .query(&deps.querier, memory_addr.clone(), key.clone())?
            .ok_or_else(|| {
                StdError::generic_err(format!("contract {} not found in memory", key))
            })?;
        contracts.insert(key.clone(), result);
    }
    Ok(contracts)
}

/// Query single contract address from mem
#[inline(always)]
fn query_contract_from_mem(deps: Deps, memory_addr: &Addr, key: &ContractEntry) -> StdResult<Addr> {
    let result: Addr = CONTRACT_ADDRESSES
        .query(&deps.querier, memory_addr.clone(), key.clone())?
        .ok_or_else(|| StdError::generic_err(format!("contract {} not found in memory", key)))?;
    // Addresses are checked when stored.
    Ok(Addr::unchecked(result))
}
