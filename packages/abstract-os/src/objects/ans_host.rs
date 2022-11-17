use std::collections::BTreeMap;

use cosmwasm_std::{Addr, QuerierWrapper, StdError, StdResult};

use cw_asset::AssetInfo;

use crate::ans_host::state::{ASSET_ADDRESSES, CHANNELS, CONTRACT_ADDRESSES};

use super::{asset_entry::AssetEntry, contract_entry::ContractEntry, ChannelEntry};

/// Struct that stores the ans-host contract address.
/// Implements `AbstractNameSystem` feature
#[cosmwasm_schema::cw_serde]
pub struct AnsHost {
    /// Address of the ans_host contract
    pub address: Addr,
}

impl AnsHost {
    /// Raw Query to AnsHost contract
    pub fn query_contracts(
        &self,
        querier: &QuerierWrapper,
        contracts: Vec<ContractEntry>,
    ) -> StdResult<BTreeMap<ContractEntry, Addr>> {
        let mut resolved_contracts: BTreeMap<ContractEntry, Addr> = BTreeMap::new();

        // Query over keys
        for key in contracts.into_iter() {
            let result: Addr = CONTRACT_ADDRESSES
                .query(querier, self.address.clone(), key.clone())?
                .ok_or_else(|| {
                    StdError::generic_err(format!("contract {} not found in ans_host", key))
                })?;
            resolved_contracts.insert(key, result);
        }
        Ok(resolved_contracts)
    }

    /// Raw query of a single contract Addr
    pub fn query_contract(
        &self,
        querier: &QuerierWrapper,
        contract: &ContractEntry,
    ) -> StdResult<Addr> {
        let result: Addr = CONTRACT_ADDRESSES
            .query(querier, self.address.clone(), contract.clone())?
            .ok_or_else(|| {
                StdError::generic_err(format!("contract {} not found in ans_host", contract))
            })?;
        // Addresses are checked when stored.
        Ok(Addr::unchecked(result))
    }

    /// Raw Query to AnsHost contract
    pub fn query_assets(
        &self,
        querier: &QuerierWrapper,
        assets: Vec<AssetEntry>,
    ) -> StdResult<BTreeMap<AssetEntry, AssetInfo>> {
        let mut resolved_assets: BTreeMap<AssetEntry, AssetInfo> = BTreeMap::new();

        for asset in assets.into_iter() {
            let result = ASSET_ADDRESSES
                .query(querier, self.address.clone(), asset.clone())?
                .ok_or_else(|| {
                    StdError::generic_err(format!("asset {} not found in ans_host", &asset))
                })?;
            resolved_assets.insert(asset, result);
        }
        Ok(resolved_assets)
    }

    /// Raw query of a single AssetInfo
    pub fn query_asset(
        &self,
        querier: &QuerierWrapper,
        asset: &AssetEntry,
    ) -> StdResult<AssetInfo> {
        let result = ASSET_ADDRESSES
            .query(querier, self.address.clone(), asset.clone())?
            .ok_or_else(|| {
                StdError::generic_err(format!("asset {} not found in ans_host", &asset))
            })?;
        Ok(result)
    }

    /// Raw query of a single channel Addr
    pub fn query_channel(
        &self,
        querier: &QuerierWrapper,
        channel: &ChannelEntry,
    ) -> StdResult<String> {
        let result: String = CHANNELS
            .query(querier, self.address.clone(), channel.clone())?
            .ok_or_else(|| {
                StdError::generic_err(format!("channel {} not found in ans_host", channel))
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
