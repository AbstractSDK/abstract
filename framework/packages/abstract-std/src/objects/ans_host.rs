use cosmwasm_std::{Addr, Api, CanonicalAddr, QuerierWrapper, StdResult};
use cw_asset::AssetInfo;
use thiserror::Error;

use super::{AssetEntry, ChannelEntry, ContractEntry};
use crate::{
    ans_host::{
        state::{
            ASSET_ADDRESSES, ASSET_PAIRINGS, CHANNELS, CONTRACT_ADDRESSES, POOL_METADATA,
            REGISTERED_DEXES, REV_ASSET_ADDRESSES,
        },
        RegisteredDexesResponse,
    },
    native_addrs,
    objects::{DexAssetPairing, PoolMetadata, PoolReference, UniquePoolId},
};

#[derive(Error, Debug, PartialEq)]
pub enum AnsHostError {
    // contract not found
    #[error("Contract {contract} not found in ans_host {ans_host}.")]
    ContractNotFound {
        contract: ContractEntry,
        ans_host: Addr,
    },

    // asset not found
    #[error("Asset {asset} not found in ans_host {ans_host}.")]
    AssetNotFound { asset: AssetEntry, ans_host: Addr },

    // cw-asset not found
    #[error("CW Asset {asset} not found in ans_host {ans_host}.")]
    CwAssetNotFound { asset: AssetInfo, ans_host: Addr },

    // channel not found
    #[error("Channel {channel} not found in ans_host {ans_host}.")]
    ChannelNotFound {
        channel: ChannelEntry,
        ans_host: Addr,
    },

    // dex asset Pairing not found
    #[error("Asset pairing {pairing} not found in ans_host {ans_host}.")]
    DexPairingNotFound {
        pairing: DexAssetPairing,
        ans_host: Addr,
    },

    // pool metadata not found
    #[error("Pool metadata for pool {pool} not found in ans_host {ans_host}.")]
    PoolMetadataNotFound { pool: UniquePoolId, ans_host: Addr },

    #[error("Object {object} should be formatted {expected} but is {actual}")]
    FormattingError {
        object: String,
        expected: String,
        actual: String,
    },

    // Query method failed
    #[error("Query during '{method_name}' failed: {error}")]
    QueryFailed {
        method_name: String,
        error: cosmwasm_std::StdError,
    },
}

pub type AnsHostResult<T> = Result<T, AnsHostError>;

/// Struct that stores the ans-host contract address.
/// Implements `AbstractNameService` feature
#[cosmwasm_schema::cw_serde]
pub struct AnsHost {
    /// Address of the ans_host contract
    pub address: Addr,
}

impl AnsHost {
    /// Retrieve address of the ans host
    pub fn new(api: &dyn Api) -> StdResult<Self> {
        let address = api.addr_humanize(&CanonicalAddr::from(native_addrs::ANS_ADDR))?;
        Ok(Self { address })
    }
    /// Raw Query to AnsHost contract
    pub fn query_contracts(
        &self,
        querier: &QuerierWrapper,
        contracts: &[ContractEntry],
    ) -> AnsHostResult<Vec<Addr>> {
        let mut resolved_contracts: Vec<Addr> = Vec::new();
        // Query over keys
        for key in contracts.iter() {
            let result = self.query_contract(querier, key)?;
            resolved_contracts.push(result);
        }
        Ok(resolved_contracts)
    }

    /// Raw query of a single contract Addr
    #[function_name::named]
    pub fn query_contract(
        &self,
        querier: &QuerierWrapper,
        contract: &ContractEntry,
    ) -> AnsHostResult<Addr> {
        let result: Addr = CONTRACT_ADDRESSES
            .query(querier, self.address.clone(), contract)
            .map_err(|error| AnsHostError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?
            .ok_or_else(|| AnsHostError::ContractNotFound {
                contract: contract.clone(),
                ans_host: self.address.clone(),
            })?;
        Ok(result)
    }

    /// Raw Query to AnsHost contract
    pub fn query_assets(
        &self,
        querier: &QuerierWrapper,
        assets: &[AssetEntry],
    ) -> AnsHostResult<Vec<AssetInfo>> {
        let mut resolved_assets = Vec::new();

        for asset in assets.iter() {
            let result = self.query_asset(querier, asset)?;
            resolved_assets.push(result);
        }
        Ok(resolved_assets)
    }

    /// Raw query of a single AssetInfo
    #[function_name::named]
    pub fn query_asset(
        &self,
        querier: &QuerierWrapper,
        asset: &AssetEntry,
    ) -> AnsHostResult<AssetInfo> {
        let result = ASSET_ADDRESSES
            .query(querier, self.address.clone(), asset)
            .map_err(|error| AnsHostError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?
            .ok_or_else(|| AnsHostError::AssetNotFound {
                asset: asset.clone(),
                ans_host: self.address.clone(),
            })?;
        Ok(result)
    }

    /// Raw Query to AnsHost contract
    pub fn query_assets_reverse(
        &self,
        querier: &QuerierWrapper,
        assets: &[AssetInfo],
    ) -> AnsHostResult<Vec<AssetEntry>> {
        // AssetInfo does not implement PartialEq, so we can't use a Vec
        let mut resolved_assets = vec![];

        for asset in assets.iter() {
            let result = self.query_asset_reverse(querier, asset)?;
            resolved_assets.push(result);
        }
        Ok(resolved_assets)
    }

    /// Raw query of a single AssetEntry
    #[function_name::named]
    pub fn query_asset_reverse(
        &self,
        querier: &QuerierWrapper,
        asset: &AssetInfo,
    ) -> AnsHostResult<AssetEntry> {
        let result = REV_ASSET_ADDRESSES
            .query(querier, self.address.clone(), asset)
            .map_err(|error| AnsHostError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?
            .ok_or_else(|| AnsHostError::CwAssetNotFound {
                asset: asset.clone(),
                ans_host: self.address.clone(),
            })?;
        Ok(result)
    }

    /// Raw query of a single channel Addr
    #[function_name::named]
    pub fn query_channel(
        &self,
        querier: &QuerierWrapper,
        channel: &ChannelEntry,
    ) -> AnsHostResult<String> {
        let result: String = CHANNELS
            .query(querier, self.address.clone(), channel)
            .map_err(|error| AnsHostError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?
            .ok_or_else(|| AnsHostError::ChannelNotFound {
                channel: channel.clone(),
                ans_host: self.address.clone(),
            })?;
        // Addresses are checked when stored.
        Ok(result)
    }

    /// Raw query of a single asset pairing
    #[function_name::named]
    pub fn query_asset_pairing(
        &self,
        querier: &QuerierWrapper,
        dex_asset_pairing: &DexAssetPairing,
    ) -> AnsHostResult<Vec<PoolReference>> {
        let result: Vec<PoolReference> = ASSET_PAIRINGS
            .query(querier, self.address.clone(), dex_asset_pairing)
            .map_err(|error| AnsHostError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?
            .ok_or_else(|| AnsHostError::DexPairingNotFound {
                pairing: dex_asset_pairing.clone(),
                ans_host: self.address.clone(),
            })?;
        Ok(result)
    }

    #[function_name::named]
    pub fn query_pool_metadata(
        &self,
        querier: &QuerierWrapper,
        pool_id: UniquePoolId,
    ) -> AnsHostResult<PoolMetadata> {
        let result: PoolMetadata = POOL_METADATA
            .query(querier, self.address.clone(), pool_id)
            .map_err(|error| AnsHostError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?
            .ok_or_else(|| AnsHostError::PoolMetadataNotFound {
                pool: pool_id,
                ans_host: self.address.clone(),
            })?;
        Ok(result)
    }

    #[function_name::named]
    pub fn query_registered_dexes(
        &self,
        querier: &QuerierWrapper,
    ) -> AnsHostResult<RegisteredDexesResponse> {
        let dexes = REGISTERED_DEXES
            .query(querier, self.address.clone())
            .map_err(|error| AnsHostError::QueryFailed {
                method_name: function_name!().to_owned(),
                error,
            })?;
        Ok(RegisteredDexesResponse { dexes })
    }
}
