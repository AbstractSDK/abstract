use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Deps, StdError, StdResult};

use cw_asset::AssetInfo;

use crate::memory::state::{ASSET_ADDRESSES, CHANNELS, CONTRACT_ADDRESSES};

use super::{asset_entry::AssetEntry, contract_entry::ContractEntry, ChannelEntry};

/// Struct that provides easy in-contract memory querying.
#[cosmwasm_schema::cw_serde]
pub struct Memory {
    /// Address of the memory contract
    pub address: Addr,
}

impl Memory {
    /// Raw Query to Memory contract
    pub fn query_contracts(
        &self,
        deps: Deps,
        contracts: Vec<ContractEntry>,
    ) -> StdResult<BTreeMap<ContractEntry, Addr>> {
        let mut resolved_contracts: BTreeMap<ContractEntry, Addr> = BTreeMap::new();

        // Query over keys
        for key in contracts.into_iter() {
            let result: Addr = CONTRACT_ADDRESSES
                .query(&deps.querier, self.address.clone(), key.clone())?
                .ok_or_else(|| {
                    StdError::generic_err(format!("contract {} not found in memory", key))
                })?;
            resolved_contracts.insert(key, result);
        }
        Ok(resolved_contracts)
    }

    /// Raw query of a single contract Addr
    pub fn query_contract(&self, deps: Deps, contract: &ContractEntry) -> StdResult<Addr> {
        let result: Addr = CONTRACT_ADDRESSES
            .query(&deps.querier, self.address.clone(), contract.clone())?
            .ok_or_else(|| {
                StdError::generic_err(format!("contract {} not found in memory", contract))
            })?;
        // Addresses are checked when stored.
        Ok(Addr::unchecked(result))
    }

    /// Raw Query to Memory contract
    pub fn query_assets(
        &self,
        deps: Deps,
        assets: Vec<AssetEntry>,
    ) -> StdResult<BTreeMap<AssetEntry, AssetInfo>> {
        let mut resolved_assets: BTreeMap<AssetEntry, AssetInfo> = BTreeMap::new();

        for asset in assets.into_iter() {
            let result = ASSET_ADDRESSES
                .query(&deps.querier, self.address.clone(), asset.clone())?
                .ok_or_else(|| {
                    StdError::generic_err(format!("asset {} not found in memory", &asset))
                })?;
            resolved_assets.insert(asset, result);
        }
        Ok(resolved_assets)
    }

    /// Raw query of a single AssetInfo
    pub fn query_asset(&self, deps: Deps, asset: &AssetEntry) -> StdResult<AssetInfo> {
        let result = ASSET_ADDRESSES
            .query(&deps.querier, self.address.clone(), asset.clone())?
            .ok_or_else(|| {
                StdError::generic_err(format!("asset {} not found in memory", &asset))
            })?;
        Ok(result)
    }

    /// Raw query of a single channel Addr
    pub fn query_channel(&self, deps: Deps, channel: &ChannelEntry) -> StdResult<String> {
        let result: String = CHANNELS
            .query(&deps.querier, self.address.clone(), channel.clone())?
            .ok_or_else(|| {
                StdError::generic_err(format!("channel {} not found in memory", channel))
            })?;
        // Addresses are checked when stored.
        Ok(result)
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
