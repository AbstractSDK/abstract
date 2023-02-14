use super::{asset_entry::AssetEntry, contract_entry::ContractEntry, ChannelEntry};
use crate::ans_host::state::{
    ASSET_ADDRESSES, ASSET_PAIRINGS, CHANNELS, CONTRACT_ADDRESSES, POOL_METADATA,
    REV_ASSET_ADDRESSES,
};
use crate::objects::{DexAssetPairing, PoolMetadata, PoolReference, UniquePoolId};
use crate::AbstractResult;
use cosmwasm_std::{Addr, QuerierWrapper, StdError};
use cw_asset::AssetInfo;
use std::collections::BTreeMap;

/// Struct that stores the ans-host contract address.
/// Implements `AbstractNameService` feature
#[cosmwasm_schema::cw_serde]
pub struct AnsHost {
    /// Address of the ans_host contract
    pub address: Addr,
}

impl AnsHost {
    /// Create a new ans_host instance with the given address.
    pub fn new(address: Addr) -> Self {
        Self { address }
    }
    /// Raw Query to AnsHost contract
    pub fn query_contracts(
        &self,
        querier: &QuerierWrapper,
        contracts: Vec<ContractEntry>,
    ) -> AbstractResult<BTreeMap<ContractEntry, Addr>> {
        let mut resolved_contracts: BTreeMap<ContractEntry, Addr> = BTreeMap::new();

        // Query over keys
        for key in contracts.into_iter() {
            let result = self.query_contract(querier, &key)?;
            resolved_contracts.insert(key, result);
        }
        Ok(resolved_contracts)
    }

    /// Raw query of a single contract Addr
    pub fn query_contract(
        &self,
        querier: &QuerierWrapper,
        contract: &ContractEntry,
    ) -> AbstractResult<Addr> {
        let result: Addr = CONTRACT_ADDRESSES
            .query(querier, self.address.clone(), contract)?
            .ok_or_else(|| {
                StdError::generic_err(format!("contract {contract} not found in ans_host"))
            })?;
        // Addresses are checked when stored.
        Ok(Addr::unchecked(result))
    }

    /// Raw Query to AnsHost contract
    pub fn query_assets(
        &self,
        querier: &QuerierWrapper,
        assets: Vec<AssetEntry>,
    ) -> AbstractResult<BTreeMap<AssetEntry, AssetInfo>> {
        let mut resolved_assets: BTreeMap<AssetEntry, AssetInfo> = BTreeMap::new();

        for asset in assets.into_iter() {
            let result = self.query_asset(querier, &asset)?;
            resolved_assets.insert(asset, result);
        }
        Ok(resolved_assets)
    }

    /// Raw query of a single AssetInfo
    pub fn query_asset(
        &self,
        querier: &QuerierWrapper,
        asset: &AssetEntry,
    ) -> AbstractResult<AssetInfo> {
        let result = ASSET_ADDRESSES
            .query(querier, self.address.clone(), asset)?
            .ok_or_else(|| {
                StdError::generic_err(format!("asset {} not found in ans_host", &asset))
            })?;
        Ok(result)
    }

    /// Raw Query to AnsHost contract
    pub fn query_assets_reverse(
        &self,
        querier: &QuerierWrapper,
        assets: Vec<AssetInfo>,
    ) -> AbstractResult<Vec<AssetEntry>> {
        // AssetInfo does not implement PartialEq, so we can't use a BTreeMap
        let mut resolved_assets = vec![];

        for asset in assets.into_iter() {
            let result = self.query_asset_reverse(querier, &asset)?;
            resolved_assets.push(result);
        }
        Ok(resolved_assets)
    }

    /// Raw query of a single AssetEntry
    pub fn query_asset_reverse(
        &self,
        querier: &QuerierWrapper,
        asset: &AssetInfo,
    ) -> AbstractResult<AssetEntry> {
        let result = REV_ASSET_ADDRESSES
            .query(querier, self.address.clone(), asset)?
            .ok_or_else(|| {
                StdError::generic_err(format!("cw-asset {} not found in ans_host", &asset))
            })?;
        Ok(result)
    }

    /// Raw query of a single channel Addr
    pub fn query_channel(
        &self,
        querier: &QuerierWrapper,
        channel: &ChannelEntry,
    ) -> AbstractResult<String> {
        let result: String = CHANNELS
            .query(querier, self.address.clone(), channel)?
            .ok_or_else(|| {
                StdError::generic_err(format!("channel {channel} not found in ans_host"))
            })?;
        // Addresses are checked when stored.
        Ok(result)
    }

    /// Raw query of a single asset pairing
    pub fn query_asset_pairing(
        &self,
        querier: &QuerierWrapper,
        dex_asset_pairing: &DexAssetPairing,
    ) -> AbstractResult<Vec<PoolReference>> {
        let result: Vec<PoolReference> = ASSET_PAIRINGS
            .query(querier, self.address.clone(), dex_asset_pairing)?
            .ok_or_else(|| {
                StdError::generic_err(format!(
                    "asset pairing {dex_asset_pairing} not found in ans_host"
                ))
            })?;
        Ok(result)
    }

    pub fn query_pool_metadata(
        &self,
        querier: &QuerierWrapper,
        pool_id: &UniquePoolId,
    ) -> AbstractResult<PoolMetadata> {
        let result: PoolMetadata = POOL_METADATA
            .query(querier, self.address.clone(), *pool_id)?
            .ok_or_else(|| {
                StdError::generic_err(format!(
                    "pool metadata for pool {} not found in ans_host",
                    pool_id.as_u64()
                ))
            })?;
        Ok(result)
    }
}
