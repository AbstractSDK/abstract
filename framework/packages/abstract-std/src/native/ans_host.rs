//! # AnsHost
//!
//! `abstract_std::ans_host` stores chain-specific contract addresses.
//!
//! ## Description
//! Contract and asset addresses are stored on the ans_host contract and are retrievable trough smart or raw queries.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;
use cw_asset::{AssetInfo, AssetInfoUnchecked};

use crate::objects::{
    pool_id::UncheckedPoolAddress, pool_reference::PoolReference, AssetEntry, ChannelEntry,
    ContractEntry, DexAssetPairing, PoolMetadata, PoolType, UncheckedChannelEntry,
    UncheckedContractEntry, UniquePoolId,
};

pub type AssetPair = (AssetEntry, AssetEntry);
type DexName = String;

/// A map entry of ((asset_x, asset_y, dex) -> compound_pool_id)
pub type AssetPairingMapEntry = (DexAssetPairing, Vec<PoolReference>);
/// Map entry for assets (asset_name -> info)
pub type AssetMapEntry = (AssetEntry, AssetInfo);
/// Map entry for assets (info -> asset_name)
pub type AssetInfoMapEntry = (AssetInfo, AssetEntry);
/// Map entry for channels
pub type ChannelMapEntry = (ChannelEntry, String);
/// Map entry for contracts (contract -> address)
pub type ContractMapEntry = (ContractEntry, Addr);
/// A map entry of (unique_pool_id -> pool_metadata)
pub type PoolMetadataMapEntry = (UniquePoolId, PoolMetadata);

/// AnsHost state details
pub mod state {
    use cosmwasm_std::Addr;
    use cw_asset::AssetInfo;
    use cw_storage_plus::{Item, Map};

    use crate::{
        ans_host::{DexAssetPairing, DexName, UniquePoolId},
        objects::{
            pool_metadata::PoolMetadata, pool_reference::PoolReference, storage_namespaces,
            AssetEntry, ChannelEntry, ContractEntry,
        },
    };

    /// Ans host configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub next_unique_pool_id: UniquePoolId,
    }

    pub const CONFIG: Item<Config> = Item::new(storage_namespaces::CONFIG_STORAGE_KEY);
    // ANCHOR: ans_state
    /// Stores name and address of tokens and pairs
    /// LP token pairs are stored alphabetically
    pub const ASSET_ADDRESSES: Map<&AssetEntry, AssetInfo> =
        Map::new(storage_namespaces::ans_host::ASSET_ADDRESSES);
    pub const REV_ASSET_ADDRESSES: Map<&AssetInfo, AssetEntry> =
        Map::new(storage_namespaces::ans_host::REV_ASSET_ADDRESSES);

    /// Stores contract addresses
    pub const CONTRACT_ADDRESSES: Map<&ContractEntry, Addr> =
        Map::new(storage_namespaces::ans_host::CONTRACT_ADDRESSES);

    /// stores channel-ids
    pub const CHANNELS: Map<&ChannelEntry, String> =
        Map::new(storage_namespaces::ans_host::CHANNELS);

    /// Stores the registered dex names
    pub const REGISTERED_DEXES: Item<Vec<DexName>> =
        Item::new(storage_namespaces::ans_host::REGISTERED_DEXES);

    /// Stores the asset pairing entries to their pool ids
    /// (asset1, asset2, dex_name) -> {id: uniqueId, pool_id: poolId}
    pub const ASSET_PAIRINGS: Map<&DexAssetPairing, Vec<PoolReference>> =
        Map::new(storage_namespaces::ans_host::ASSET_PAIRINGS);

    /// Stores the metadata for the pools using the unique pool id as the key
    pub const POOL_METADATA: Map<UniquePoolId, PoolMetadata> =
        Map::new(storage_namespaces::ans_host::POOL_METADATA);
    // ANCHOR_END: ans_state
}

/// AnsHost Instantiate msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
}

/// AnsHost Execute msg
#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// Updates the contract addressbook
    UpdateContractAddresses {
        // Contracts to update or add
        to_add: Vec<(UncheckedContractEntry, String)>,
        // Contracts to remove
        to_remove: Vec<UncheckedContractEntry>,
    },
    /// Updates the Asset addressbook
    UpdateAssetAddresses {
        // Assets to update or add
        to_add: Vec<(String, AssetInfoUnchecked)>,
        // Assets to remove
        to_remove: Vec<String>,
    },
    /// Updates the Asset addressbook
    UpdateChannels {
        // Assets to update or add
        to_add: Vec<(UncheckedChannelEntry, String)>,
        // Assets to remove
        to_remove: Vec<UncheckedChannelEntry>,
    },
    /// Registers a dex
    UpdateDexes {
        // Dexes to add
        to_add: Vec<String>,
        // Dexes to remove
        to_remove: Vec<String>,
    },
    /// Update the pools
    UpdatePools {
        // Pools to update or add
        to_add: Vec<(UncheckedPoolAddress, PoolMetadata)>,
        // Pools to remove
        to_remove: Vec<UniquePoolId>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct AssetPairingFilter {
    /// Filter by asset pair
    pub asset_pair: Option<AssetPair>,
    /// Filter by dex
    pub dex: Option<String>,
}

/// UNUSED - stub for future use
#[cosmwasm_schema::cw_serde]
pub struct ContractFilter {}

/// UNUSED - stub for future use
#[cosmwasm_schema::cw_serde]
pub struct ChannelFilter {}

/// UNUSED - stub for future use
#[cosmwasm_schema::cw_serde]
pub struct AssetFilter {}

/// UNUSED - stub for future use
#[cosmwasm_schema::cw_serde]
pub struct AssetInfoFilter {}

/// Filter on the pool metadatas
#[cosmwasm_schema::cw_serde]
#[derive(Default)]
pub struct PoolMetadataFilter {
    /// Filter by pool type
    pub pool_type: Option<PoolType>,
    // /// Filter by pool status
    // pub pool_status: Option<PoolStatus>,
}

/// AnsHost smart-query
#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Query the config
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Queries assets based on name
    /// returns [`AssetsResponse`]
    #[returns(AssetsResponse)]
    Assets {
        // Names of assets to query
        names: Vec<String>,
    },
    /// Page over assets
    /// returns [`AssetListResponse`]
    #[returns(AssetListResponse)]
    AssetList {
        filter: Option<AssetFilter>,
        start_after: Option<String>,
        limit: Option<u8>,
    },
    /// Queries assets based on address
    /// returns [`AssetInfosResponse`]
    #[returns(AssetInfosResponse)]
    AssetInfos {
        // Addresses of assets to query
        infos: Vec<AssetInfoUnchecked>,
    },
    /// Page over asset infos
    /// returns [`AssetInfoListResponse`]
    #[returns(AssetInfoListResponse)]
    AssetInfoList {
        filter: Option<AssetInfoFilter>,
        start_after: Option<AssetInfoUnchecked>,
        limit: Option<u8>,
    },
    /// Queries contracts based on name
    /// returns [`ContractsResponse`]
    #[returns(ContractsResponse)]
    Contracts {
        // Project and contract names of contracts to query
        entries: Vec<ContractEntry>,
    },
    /// Page over contracts
    /// returns [`ContractListResponse`]
    #[returns(ContractListResponse)]
    ContractList {
        filter: Option<ContractFilter>,
        start_after: Option<ContractEntry>,
        limit: Option<u8>,
    },
    /// Queries contracts based on name
    /// returns [`ChannelsResponse`]
    #[returns(ChannelsResponse)]
    Channels {
        // Project and contract names of contracts to query
        entries: Vec<ChannelEntry>,
    },
    /// Page over contracts
    /// returns [`ChannelListResponse`]
    #[returns(ChannelListResponse)]
    ChannelList {
        filter: Option<ChannelFilter>,
        start_after: Option<ChannelEntry>,
        limit: Option<u8>,
    },
    /// Retrieve the registered dexes
    /// returns [`RegisteredDexesResponse`]
    #[returns(RegisteredDexesResponse)]
    RegisteredDexes {},
    /// Retrieve the pools with the specified keys
    /// returns [`PoolsResponse`]
    #[returns(PoolsResponse)]
    Pools { pairings: Vec<DexAssetPairing> },
    /// Retrieve the (optionally-filtered) list of pools.
    /// returns [`PoolAddressListResponse`]
    #[returns(PoolAddressListResponse)]
    PoolList {
        filter: Option<AssetPairingFilter>,
        start_after: Option<DexAssetPairing>,
        limit: Option<u8>,
    },
    /// Get the pool metadatas for given pool ids
    /// returns [`PoolMetadatasResponse`]
    #[returns(PoolMetadatasResponse)]
    PoolMetadatas { ids: Vec<UniquePoolId> },
    /// Retrieve the (optionally-filtered) list of pool metadatas
    /// returns [`PoolMetadataListResponse`]
    #[returns(PoolMetadataListResponse)]
    PoolMetadataList {
        filter: Option<PoolMetadataFilter>,
        start_after: Option<UniquePoolId>,
        limit: Option<u8>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub next_unique_pool_id: UniquePoolId,
}
/// Query response
#[cosmwasm_schema::cw_serde]
pub struct AssetsResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<AssetMapEntry>,
}

/// Query response
#[cosmwasm_schema::cw_serde]
pub struct AssetListResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<AssetMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct AssetInfosResponse {
    /// Assets (assetinfo, name)
    pub infos: Vec<AssetInfoMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct AssetInfoListResponse {
    /// Assets (assetinfo, name)
    pub infos: Vec<AssetInfoMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct ContractsResponse {
    /// Contracts (name, address)
    pub contracts: Vec<ContractMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct ContractListResponse {
    /// Contracts (name, address)
    pub contracts: Vec<ContractMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChannelsResponse {
    pub channels: Vec<ChannelMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChannelListResponse {
    pub channels: Vec<ChannelMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct RegisteredDexesResponse {
    pub dexes: Vec<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct PoolAddressListResponse {
    pub pools: Vec<AssetPairingMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct PoolsResponse {
    pub pools: Vec<AssetPairingMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct PoolMetadatasResponse {
    pub metadatas: Vec<PoolMetadataMapEntry>,
}

#[cosmwasm_schema::cw_serde]
pub struct PoolMetadataListResponse {
    pub metadatas: Vec<PoolMetadataMapEntry>,
}
