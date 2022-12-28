use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult, Storage};

use abstract_os::ans_host::state::{ASSET_PAIRINGS, POOL_METADATA};
use abstract_os::ans_host::{
    AssetPairingFilter, AssetPairingMapEntry, PoolIdListResponse, PoolMetadataFilter,
    PoolMetadataListResponse, PoolMetadataMapEntry, PoolMetadatasResponse, PoolsResponse,
    RegisteredDexesResponse,
};
use abstract_os::dex::DexName;
use abstract_os::objects::pool_metadata::PoolMetadata;
use abstract_os::objects::pool_reference::PoolReference;
use abstract_os::objects::{DexAssetPairing, UniquePoolId};
use abstract_os::{
    ans_host::{
        state::{ASSET_ADDRESSES, CHANNELS, CONTRACT_ADDRESSES, REGISTERED_DEXES},
        AssetListResponse, AssetsResponse, ChannelListResponse, ChannelsResponse,
        ContractListResponse, ContractsResponse,
    },
    objects::{AssetEntry, ChannelEntry, ContractEntry},
};
use cw_asset::AssetInfo;
use cw_storage_plus::Bound;

pub(crate) const DEFAULT_LIMIT: u8 = 15;
pub(crate) const MAX_LIMIT: u8 = 25;

pub fn query_assets(deps: Deps, _env: Env, asset_names: Vec<String>) -> StdResult<Binary> {
    let assets: Vec<AssetEntry> = asset_names
        .iter()
        .map(|name| name.as_str().into())
        .collect();
    let res: Result<Vec<(AssetEntry, AssetInfo)>, _> = ASSET_ADDRESSES
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| assets.contains(&e.as_ref().unwrap().0))
        .collect();
    to_binary(&AssetsResponse { assets: res? })
}

pub fn query_contract(deps: Deps, _env: Env, names: Vec<ContractEntry>) -> StdResult<Binary> {
    let res: Result<Vec<(ContractEntry, Addr)>, _> = CONTRACT_ADDRESSES
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| names.contains(&e.as_ref().unwrap().0))
        .collect();

    to_binary(&ContractsResponse {
        contracts: res?.into_iter().map(|(x, a)| (x, a.to_string())).collect(),
    })
}

pub fn query_channel(deps: Deps, _env: Env, names: Vec<ChannelEntry>) -> StdResult<Binary> {
    let res: Result<Vec<(ChannelEntry, String)>, _> = CHANNELS
        .range(deps.storage, None, None, Order::Ascending)
        .filter(|e| names.contains(&e.as_ref().unwrap().0))
        .collect();

    to_binary(&ChannelsResponse { channels: res? })
}

pub fn query_asset_list(
    deps: Deps,
    last_asset_name: Option<String>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_asset_name.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(AssetEntry, AssetInfo)>, _> = ASSET_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    to_binary(&AssetListResponse { assets: res? })
}

pub fn query_contract_list(
    deps: Deps,
    last_contract: Option<ContractEntry>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_contract.map(Bound::exclusive);

    let res: Result<Vec<(ContractEntry, Addr)>, _> = CONTRACT_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();
    to_binary(&ContractListResponse {
        contracts: res?.into_iter().map(|(x, a)| (x, a.to_string())).collect(),
    })
}

pub fn query_channel_list(
    deps: Deps,
    last_channel: Option<ChannelEntry>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_channel.map(Bound::exclusive);

    let res: Result<Vec<(ChannelEntry, String)>, _> = CHANNELS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();
    to_binary(&ChannelListResponse { channels: res? })
}

pub fn query_registered_dexes(deps: Deps, _env: Env) -> StdResult<Binary> {
    let dexes = REGISTERED_DEXES.load(deps.storage)?;

    to_binary(&RegisteredDexesResponse { dexes })
}

pub fn list_pool_entries(
    deps: Deps,
    filter: Option<AssetPairingFilter>,
    page_token: Option<DexAssetPairing>,
    page_size: Option<u8>,
) -> StdResult<Binary> {
    let page_size = page_size.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let (asset_pair_filter, dex_filter) = match filter {
        Some(AssetPairingFilter { asset_pair, dex }) => (asset_pair, dex),
        None => (None, None),
    };

    let full_key_provided = asset_pair_filter.is_some() && dex_filter.is_some();

    let entry_list: Vec<AssetPairingMapEntry> = if full_key_provided {
        // We have the full key, so load the entry
        let (asset_x, asset_y) = asset_pair_filter.unwrap();
        let key = DexAssetPairing::new(&asset_x, &asset_y, &dex_filter.unwrap());
        let entry = load_asset_pairing_entry(deps.storage, key)?;
        // Add the result to a vec
        vec![entry]
    } else if let Some((asset_x, asset_y)) = asset_pair_filter {
        let start_bound = page_token.map(|pairing| Bound::exclusive(pairing.dex()));

        // We can use the prefix to load all the entries for the asset pair
        let res: Result<Vec<(DexName, Vec<PoolReference>)>, _> = ASSET_PAIRINGS
            .prefix((asset_x.clone(), asset_y.clone()))
            .range(deps.storage, start_bound, None, Order::Ascending)
            .take(page_size)
            .collect();

        // Re add the key prefix, since only the dex is returned as a key
        let matched: Vec<AssetPairingMapEntry> = res?
            .into_iter()
            .map(|(dex, ids)| (DexAssetPairing::new(&asset_x, &asset_y, &dex), ids))
            .collect();

        matched
    } else {
        let start_bound: Option<Bound<DexAssetPairing>> = page_token.map(Bound::exclusive);

        // We have no filter, so load all the entries
        let res: Result<Vec<AssetPairingMapEntry>, _> = ASSET_PAIRINGS
            .range(deps.storage, start_bound, None, Order::Ascending)
            .filter(|e| {
                let pairing = &e.as_ref().unwrap().0;
                dex_filter.as_ref().map_or(true, |f| f == pairing.dex())
            })
            // TODO: is this necessary?
            .map(|e| e.map(|(k, v)| (k, v)))
            .collect();
        res?
    };

    to_binary(&PoolIdListResponse { pools: entry_list })
}

/// Query the pool ids based on the actual keys
pub fn query_pool_entries(deps: Deps, keys: Vec<DexAssetPairing>) -> StdResult<Binary> {
    let mut entries: Vec<AssetPairingMapEntry> = vec![];
    for key in keys.into_iter() {
        let entry = load_asset_pairing_entry(deps.storage, key)?;

        entries.push(entry);
    }

    to_binary(&PoolsResponse { pools: entries })
}

/// Loads a given key from the asset pairings store and returns the ENTRY
fn load_asset_pairing_entry(
    storage: &dyn Storage,
    key: DexAssetPairing,
) -> StdResult<AssetPairingMapEntry> {
    let value = ASSET_PAIRINGS.load(storage, key.clone())?;
    Ok((key, value))
}

pub fn query_pool_metadatas(deps: Deps, keys: Vec<UniquePoolId>) -> StdResult<Binary> {
    let mut entries: Vec<PoolMetadataMapEntry> = vec![];
    for key in keys.into_iter() {
        let entry = load_pool_metadata_entry(deps.storage, key)?;

        entries.push(entry);
    }

    to_binary(&PoolMetadatasResponse { metadatas: entries })
}

pub fn list_pool_metadata_entries(
    deps: Deps,
    filter: Option<PoolMetadataFilter>,
    page_token: Option<UniquePoolId>,
    page_size: Option<u8>,
) -> StdResult<Binary> {
    let page_size = page_size.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = page_token.map(Bound::exclusive);

    let pool_type_filter = match filter {
        Some(PoolMetadataFilter { pool_type }) => pool_type,
        None => None,
    };

    let res: Result<Vec<(UniquePoolId, PoolMetadata)>, _> = POOL_METADATA
        // If the asset_pair_filter is provided, we must use that prefix...
        .range(deps.storage, start_bound, None, Order::Ascending)
        .filter(|e| {
            let pool_type = &e.as_ref().unwrap().1.pool_type;
            pool_type_filter.as_ref().map_or(true, |f| f == pool_type)
        })
        .take(page_size)
        .map(|e| e.map(|(k, v)| (k, v)))
        .collect();

    to_binary(&PoolMetadataListResponse { metadatas: res? })
}

/// Loads a given key from the asset pairings store and returns the ENTRY
fn load_pool_metadata_entry(
    storage: &dyn Storage,
    key: UniquePoolId,
) -> StdResult<PoolMetadataMapEntry> {
    let value = POOL_METADATA.load(storage, key)?;
    Ok((key, value))
}
