use abstract_core::{
    ans_host::{
        state::{
            Config, ASSET_ADDRESSES, ASSET_PAIRINGS, CHANNELS, CONFIG, CONTRACT_ADDRESSES,
            POOL_METADATA, REGISTERED_DEXES, REV_ASSET_ADDRESSES,
        },
        AssetInfoListResponse, AssetInfoMapEntry, AssetInfosResponse, AssetListResponse,
        AssetMapEntry, AssetPairingFilter, AssetPairingMapEntry, AssetsResponse,
        ChannelListResponse, ChannelMapEntry, ChannelsResponse, ConfigResponse,
        ContractListResponse, ContractMapEntry, ContractsResponse, PoolAddressListResponse,
        PoolMetadataFilter, PoolMetadataListResponse, PoolMetadataMapEntry, PoolMetadatasResponse,
        PoolsResponse, RegisteredDexesResponse,
    },
    objects::{
        AssetEntry, ChannelEntry, ContractEntry, DexAssetPairing, DexName, PoolMetadata,
        PoolReference, UniquePoolId,
    },
};
use abstract_sdk::cw_helpers::load_many;
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdError, StdResult, Storage};
use cw_asset::AssetInfoUnchecked;
use cw_storage_plus::Bound;

pub(crate) const DEFAULT_LIMIT: u8 = 15;
pub(crate) const MAX_LIMIT: u8 = 25;

pub fn query_config(deps: Deps) -> StdResult<Binary> {
    let Config {
        next_unique_pool_id,
    } = CONFIG.load(deps.storage)?;

    let res = ConfigResponse {
        next_unique_pool_id,
    };

    to_binary(&res)
}

pub fn query_assets(deps: Deps, _env: Env, keys: Vec<String>) -> StdResult<Binary> {
    let keys: Vec<AssetEntry> = keys.into_iter().map(|name| name.as_str().into()).collect();

    let assets = load_many(ASSET_ADDRESSES, deps.storage, keys.iter().collect())?;

    to_binary(&AssetsResponse {
        assets: assets.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
    })
}

pub fn query_asset_list(
    deps: Deps,
    last_asset_name: Option<String>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let entry = last_asset_name.map(AssetEntry::from);
    let start_bound = entry.as_ref().map(Bound::exclusive);

    let res: Result<Vec<AssetMapEntry>, _> = ASSET_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    to_binary(&AssetListResponse { assets: res? })
}

pub fn query_asset_infos(
    deps: Deps,
    _env: Env,
    keys: Vec<AssetInfoUnchecked>,
) -> StdResult<Binary> {
    let keys = keys
        .into_iter()
        .map(|info| {
            info.check(deps.api, None)
                .map_err(|e| StdError::generic_err(e.to_string()))
        })
        .collect::<StdResult<Vec<_>>>()?;

    let infos = load_many(REV_ASSET_ADDRESSES, deps.storage, keys.iter().collect())?;

    to_binary(&AssetInfosResponse {
        infos: infos.into_iter().map(|(k, v)| (k.to_owned(), v)).collect(),
    })
}

pub fn query_asset_info_list(
    deps: Deps,
    last_asset_info: Option<AssetInfoUnchecked>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let asset_info = last_asset_info
        .map(|info| {
            info.check(deps.api, None)
                .map_err(|e| StdError::generic_err(e.to_string()))
        })
        .transpose()?;
    let start_bound = asset_info.as_ref().map(Bound::exclusive);

    let res: Result<Vec<AssetInfoMapEntry>, _> = REV_ASSET_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    to_binary(&AssetInfoListResponse { infos: res? })
}

pub fn query_contract(deps: Deps, _env: Env, keys: Vec<&ContractEntry>) -> StdResult<Binary> {
    let contracts = load_many(CONTRACT_ADDRESSES, deps.storage, keys)?;

    to_binary(&ContractsResponse {
        contracts: contracts
            .into_iter()
            .map(|(x, a)| (x.to_owned(), a))
            .collect(),
    })
}

pub fn query_channels(deps: Deps, _env: Env, keys: Vec<&ChannelEntry>) -> StdResult<Binary> {
    let channels = load_many(CHANNELS, deps.storage, keys)?;

    to_binary(&ChannelsResponse {
        channels: channels
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect(),
    })
}

pub fn query_contract_list(
    deps: Deps,
    last_contract: Option<ContractEntry>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_contract.as_ref().map(Bound::exclusive);

    let res: Result<Vec<ContractMapEntry>, _> = CONTRACT_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    to_binary(&ContractListResponse { contracts: res? })
}

pub fn query_channel_list(
    deps: Deps,
    last_channel: Option<ChannelEntry>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_channel.as_ref().map(Bound::exclusive);

    let res: Result<Vec<ChannelMapEntry>, _> = CHANNELS
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
    start_after: Option<DexAssetPairing>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let (asset_pair_filter, dex_filter) = match filter {
        Some(AssetPairingFilter { asset_pair, dex }) => (asset_pair, dex),
        None => (None, None),
    };

    let full_key_provided = asset_pair_filter.is_some() && dex_filter.is_some();

    let entry_list: Vec<AssetPairingMapEntry> = if full_key_provided {
        // We have the full key, so load the entry
        let (asset_x, asset_y) = asset_pair_filter.unwrap();
        let key = DexAssetPairing::new(asset_x, asset_y, &dex_filter.unwrap());
        let entry = load_asset_pairing_entry(deps.storage, key)?;
        // Add the result to a vec
        vec![entry]
    } else if let Some((asset_x, asset_y)) = asset_pair_filter {
        let start_bound = start_after.map(|pairing| Bound::exclusive(pairing.dex()));

        // We can use the prefix to load all the entries for the asset pair
        let res: Result<Vec<(DexName, Vec<PoolReference>)>, _> = ASSET_PAIRINGS
            .prefix((&asset_x, &asset_y))
            .range(deps.storage, start_bound, None, Order::Ascending)
            .take(limit)
            .collect();

        // Re add the key prefix, since only the dex is returned as a key
        let matched: Vec<AssetPairingMapEntry> = res?
            .into_iter()
            .map(|(dex, ids)| {
                (
                    DexAssetPairing::new(asset_x.clone(), asset_y.clone(), &dex),
                    ids,
                )
            })
            .collect();

        matched
    } else {
        let start_bound: Option<Bound<&DexAssetPairing>> =
            start_after.as_ref().map(Bound::exclusive);

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

    to_binary(&PoolAddressListResponse { pools: entry_list })
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
    let value = ASSET_PAIRINGS.load(storage, &key)?;
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
    start_after: Option<UniquePoolId>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = start_after.map(Bound::exclusive);

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
        .take(limit)
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

#[cfg(test)]
mod test {
    use abstract_core::ans_host::*;
    use abstract_core::objects::PoolType;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi};
    use cosmwasm_std::{from_binary, Addr, DepsMut};

    use crate::contract;
    use crate::contract::{instantiate, AnsHostResult};
    use crate::error::AnsHostError;

    use abstract_core::objects::pool_id::PoolAddressBase;
    use cw_asset::{AssetInfo, AssetInfoUnchecked};
    use speculoos::prelude::*;

    use super::*;

    type AnsHostTestResult = Result<(), AnsHostError>;

    const TEST_CREATOR: &str = "creator";

    fn mock_init(mut deps: DepsMut) -> AnsHostResult {
        let info = mock_info(TEST_CREATOR, &[]);

        instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})
    }

    fn query_helper(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
        let res = contract::query(deps, mock_env(), msg)?;
        Ok(res)
    }

    fn query_asset_list_msg(token: String, size: usize) -> QueryMsg {
        QueryMsg::AssetList {
            start_after: (Some(token)),
            limit: (Some(size as u8)),
            filter: None,
        }
    }

    fn create_test_assets(input: Vec<(&str, &str)>, api: MockApi) -> Vec<(String, AssetInfo)> {
        let test_assets: Vec<(String, AssetInfo)> = input
            .into_iter()
            .map(|input| {
                (
                    input.0.to_string(),
                    (AssetInfoUnchecked::native(input.1.to_string()))
                        .check(&api, None)
                        .unwrap(),
                )
            })
            .collect();
        test_assets
    }

    fn create_asset_response(test_assets: Vec<(String, AssetInfo)>) -> AssetsResponse {
        let expected = AssetsResponse {
            assets: test_assets
                .iter()
                .map(|item| (item.0.clone().into(), item.1.clone()))
                .collect(),
        };
        expected
    }

    fn create_asset_list_response(test_assets: Vec<(String, AssetInfo)>) -> AssetListResponse {
        let expected = AssetListResponse {
            assets: test_assets
                .iter()
                .map(|item| (item.0.clone().into(), item.1.clone()))
                .collect(),
        };
        expected
    }

    fn create_contract_entry_and_string(
        input: Vec<(&str, &str, &str)>,
    ) -> Vec<(ContractEntry, String)> {
        let contract_entry: Vec<(ContractEntry, String)> = input
            .into_iter()
            .map(|input| {
                (
                    ContractEntry {
                        protocol: input.0.to_string().to_ascii_lowercase(),
                        contract: input.1.to_string().to_ascii_lowercase(),
                    },
                    input.2.to_string(),
                )
            })
            .collect();
        contract_entry
    }

    fn create_contract_entry(input: Vec<(&str, &str)>) -> Vec<ContractEntry> {
        let contract_entry: Vec<ContractEntry> = input
            .into_iter()
            .map(|input| ContractEntry {
                protocol: input.0.to_string().to_ascii_lowercase(),
                contract: input.1.to_string().to_ascii_lowercase(),
            })
            .collect();
        contract_entry
    }

    fn create_channel_entry_and_string(input: Vec<(&str, &str, &str)>) -> Vec<ChannelMapEntry> {
        let channel_entry: Vec<ChannelMapEntry> = input
            .into_iter()
            .map(|input| {
                (
                    ChannelEntry {
                        connected_chain: input.0.to_string().to_ascii_lowercase(),
                        protocol: input.1.to_string().to_ascii_lowercase(),
                    },
                    input.2.to_string(),
                )
            })
            .collect();
        channel_entry
    }

    fn create_channel_entry(input: Vec<(&str, &str)>) -> Vec<ChannelEntry> {
        let channel_entry: Vec<ChannelEntry> = input
            .into_iter()
            .map(|input| ChannelEntry {
                connected_chain: input.0.to_string().to_ascii_lowercase(),
                protocol: input.1.to_string().to_ascii_lowercase(),
            })
            .collect();
        channel_entry
    }

    fn create_channel_msg(input: Vec<(&str, &str)>) -> QueryMsg {
        QueryMsg::Channels {
            entries: create_channel_entry(input),
        }
    }

    fn update_asset_addresses(
        deps: DepsMut<'_>,
        to_add: Vec<(String, AssetInfo)>,
    ) -> Result<(), cosmwasm_std::StdError> {
        for (test_asset_name, test_asset_info) in to_add.into_iter() {
            let insert = |_| -> StdResult<AssetInfo> { Ok(test_asset_info) };
            ASSET_ADDRESSES.update(deps.storage, &test_asset_name.into(), insert)?;
        }
        Ok(())
    }

    fn update_contract_addresses(
        deps: DepsMut<'_>,
        to_add: Vec<(ContractEntry, String)>,
    ) -> Result<(), cosmwasm_std::StdError> {
        for (key, new_address) in to_add.into_iter() {
            let addr = deps.as_ref().api.addr_validate(&new_address)?;
            let insert = |_| -> StdResult<Addr> { Ok(addr) };
            CONTRACT_ADDRESSES.update(deps.storage, &key, insert)?;
        }
        Ok(())
    }

    fn update_channels(
        deps: DepsMut<'_>,
        to_add: Vec<ChannelMapEntry>,
    ) -> Result<(), cosmwasm_std::StdError> {
        for (key, new_channel) in to_add.into_iter() {
            // Update function for new or existing keys
            let insert = |_| -> StdResult<String> { Ok(new_channel) };
            CHANNELS.update(deps.storage, &key, insert)?;
        }
        Ok(())
    }

    fn update_registered_dexes(
        deps: DepsMut<'_>,
        to_add: Vec<String>,
    ) -> Result<(), cosmwasm_std::StdError> {
        for _dex in to_add.clone() {
            let register_dex = |mut dexes: Vec<String>| -> StdResult<Vec<String>> {
                for _dex in to_add.clone() {
                    if !dexes.contains(&_dex) {
                        dexes.push(_dex.to_ascii_lowercase());
                    }
                }
                Ok(dexes)
            };
            REGISTERED_DEXES.update(deps.storage, register_dex)?;
        }
        Ok(())
    }

    fn update_asset_pairing(
        asset_x: &str,
        asset_y: &str,
        dex: &str,
        id: u64,
        deps: DepsMut<'_>,
        api: MockApi,
    ) -> Result<(), cosmwasm_std::StdError> {
        let dex_asset_pairing =
            DexAssetPairing::new(AssetEntry::new(asset_x), AssetEntry::new(asset_y), dex);
        let _pool_ref = create_option_pool_ref(id, dex, api);
        let insert = |pool_ref: Option<Vec<PoolReference>>| -> StdResult<_> {
            let _pool_ref = pool_ref.unwrap_or_default();
            Ok(_pool_ref)
        };
        ASSET_PAIRINGS.update(deps.storage, &dex_asset_pairing, insert)?;
        Ok(())
    }

    fn create_dex_asset_pairing(asset_x: &str, asset_y: &str, dex: &str) -> DexAssetPairing {
        DexAssetPairing::new(AssetEntry::new(asset_x), AssetEntry::new(asset_y), dex)
    }

    fn create_asset_pairing_filter(
        asset_x: &str,
        asset_y: &str,
        dex: Option<String>,
    ) -> Result<AssetPairingFilter, cosmwasm_std::StdError> {
        let filter = AssetPairingFilter {
            asset_pair: Some((AssetEntry::new(asset_x), AssetEntry::new(asset_y))),
            dex,
        };
        Ok(filter)
    }

    fn create_pool_list_msg(
        filter: Option<AssetPairingFilter>,
        start_after: Option<DexAssetPairing>,
        limit: Option<u8>,
    ) -> Result<QueryMsg, cosmwasm_std::StdError> {
        let msg = QueryMsg::PoolList {
            filter,
            start_after,
            limit,
        };
        Ok(msg)
    }

    fn load_asset_pairing_into_pools_response(
        asset_x: &str,
        asset_y: &str,
        dex: &str,
        deps: DepsMut<'_>,
    ) -> Result<PoolsResponse, cosmwasm_std::StdError> {
        let asset_pairing = ASSET_PAIRINGS
            .load(
                deps.storage,
                &create_dex_asset_pairing(asset_x, asset_y, dex),
            )
            .unwrap();
        let asset_pairing = PoolsResponse {
            pools: vec![(
                create_dex_asset_pairing(asset_x, asset_y, dex),
                asset_pairing,
            )],
        };
        Ok(asset_pairing)
    }

    fn create_option_pool_ref(id: u64, pool_id: &str, api: MockApi) -> Option<Vec<PoolReference>> {
        Some(vec![PoolReference {
            unique_id: UniquePoolId::new(id),
            pool_address: PoolAddressBase::contract(pool_id).check(&api).unwrap(),
        }])
    }

    fn create_pool_metadata(dex: &str, asset_x: &str, asset_y: &str) -> PoolMetadata {
        PoolMetadata::new(
            dex,
            PoolType::Stable,
            vec![asset_x.to_string(), asset_y.to_string()],
        )
    }

    #[test]
    fn test_query_assets() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;

        // create test query data
        let test_assets = create_test_assets(vec![("bar", "bar"), ("foo", "foo")], api);
        update_asset_addresses(deps.as_mut(), test_assets)?;
        // create msg
        let msg = QueryMsg::Assets {
            names: vec!["bar".to_string(), "foo".to_string()],
        };
        // send query message
        let res: AssetsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = create_asset_response(create_test_assets(
            vec![("bar", "bar"), ("foo", "foo")],
            api,
        ));
        // Assert
        assert_that!(&res).is_equal_to(&expected);

        Ok(())
    }

    #[test]
    fn test_query_contract() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // create test query data
        let to_add = create_contract_entry_and_string(vec![("foo", "foo", "foo")]);
        update_contract_addresses(deps.as_mut(), to_add)?;
        // create, send and deserialise msg
        let msg = QueryMsg::Contracts {
            entries: create_contract_entry(vec![("foo", "foo")]),
        };
        let res: ContractsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = ContractsResponse {
            contracts: create_contract_entry_and_string(vec![("foo", "foo", "foo")])
                .into_iter()
                .map(|(a, b)| (a, Addr::unchecked(b)))
                .collect(),
        };

        // Assert
        assert_that!(&res).is_equal_to(&expected);

        Ok(())
    }

    #[test]
    fn test_query_channels() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // create test query data
        let to_add = create_channel_entry_and_string(vec![("foo", "foo", "foo")]);
        update_channels(deps.as_mut(), to_add)?;
        // create duplicate entry
        let to_add1 = create_channel_entry_and_string(vec![("foo", "foo", "foo")]);
        update_channels(deps.as_mut(), to_add1)?;

        // create and send and deserialise msg
        let msg = create_channel_msg(vec![("foo", "foo")]);
        let res: ChannelsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = ChannelsResponse {
            channels: create_channel_entry_and_string(vec![("foo", "foo", "foo")]),
        };
        // Assert
        assert_that!(&res).is_equal_to(&expected);
        // Assert no duplication
        assert!(res.channels.len() == 1_usize);
        Ok(())
    }

    #[test]
    fn test_query_asset_list() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;

        // create test query data
        let test_assets = create_test_assets(vec![("foo", "foo"), ("bar", "bar")], api);
        update_asset_addresses(deps.as_mut(), test_assets)?;

        // create second entry
        let test_assets1 = create_test_assets(vec![("foobar", "foobar")], api);
        update_asset_addresses(deps.as_mut(), test_assets1)?;

        // create duplicate entry
        let test_assets_duplicate = create_test_assets(vec![("foobar", "foobar")], api);
        update_asset_addresses(deps.as_mut(), test_assets_duplicate)?;

        // return all entries
        let msg = query_asset_list_msg("".to_string(), 42);
        let res: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // limit response to 1st result - entries are stored alphabetically
        let msg = query_asset_list_msg("".to_string(), 1);
        let res_first_entry: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // results after specified entry
        let msg = query_asset_list_msg("foo".to_string(), 1);
        let res_of_foobar: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = create_asset_list_response(create_test_assets(
            vec![("bar", "bar"), ("foo", "foo"), ("foobar", "foobar")],
            api,
        ));

        let expected_foobar =
            create_asset_list_response(create_test_assets(vec![("foobar", "foobar")], api));
        let expected_bar =
            create_asset_list_response(create_test_assets(vec![("bar", "bar")], api));

        assert_that!(res).is_equal_to(&expected);
        assert_that!(res_first_entry).is_equal_to(&expected_bar);
        assert_that!(&res_of_foobar).is_equal_to(&expected_foobar);

        Ok(())
    }
    #[test]
    fn test_query_asset_list_above_max() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;

        // create test query data
        let generate_test_assets_large = |n: usize| -> Vec<(String, String)> {
            let mut vector = vec![];
            for i in 0..n {
                let string1 = format!("foo{i}");
                let string2 = format!("foo{i}");
                vector.push((string1, string2));
            }
            vector
        };
        let test_assets_large: Vec<(String, AssetInfo)> = generate_test_assets_large(30)
            .into_iter()
            .map(|input| {
                (
                    input.0.clone(),
                    (AssetInfoUnchecked::native(input.1))
                        .check(&api, None)
                        .unwrap(),
                )
            })
            .collect();
        update_asset_addresses(deps.as_mut(), test_assets_large)?;

        let msg = query_asset_list_msg("".to_string(), 42);
        let res: AssetListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;
        assert!(res.assets.len() == 25_usize);

        // Assert that despite 30 entries the returned data is capped at the `MAX_LIMIT` of 25 results
        assert!(res.assets.len() == 25_usize);
        Ok(())
    }
    #[test]
    fn test_query_contract_list() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // create test query data
        let to_add = create_contract_entry_and_string(vec![("foo", "foo1", "foo2")]);
        update_contract_addresses(deps.as_mut(), to_add)?;

        // create second entry
        let to_add1 = create_contract_entry_and_string(vec![("bar", "bar1", "bar2")]);
        update_contract_addresses(deps.as_mut(), to_add1)?;

        // create duplicate entry
        let to_add1 = create_contract_entry_and_string(vec![("bar", "bar1", "bar2")]);
        update_contract_addresses(deps.as_mut(), to_add1)?;

        // create msgs
        let msg = QueryMsg::ContractList {
            start_after: None,
            limit: Some(42_u8),
            filter: None,
        };
        let res: ContractListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        let msg = QueryMsg::ContractList {
            start_after: Some(ContractEntry {
                protocol: "bar".to_string().to_ascii_lowercase(),
                contract: "bar1".to_string().to_ascii_lowercase(),
            }),
            limit: Some(42_u8),
            filter: None,
        };
        let res_expect_foo: ContractListResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test
        let expected = ContractListResponse {
            contracts: create_contract_entry_and_string(vec![
                ("bar", "bar1", "bar2"),
                ("foo", "foo1", "foo2"),
            ])
            .into_iter()
            .map(|(a, b)| (a, Addr::unchecked(b)))
            .collect(),
        };

        let expected_foo = ContractListResponse {
            contracts: create_contract_entry_and_string(vec![("foo", "foo1", "foo2")])
                .into_iter()
                .map(|(a, b)| (a, Addr::unchecked(b)))
                .collect(),
        };

        // Assert
        // Assert only returns unqiue data entries looping
        assert_that!(&res).is_equal_to(&expected);
        // Assert - sanity check for duplication
        assert_that!(&res_expect_foo).is_equal_to(&expected_foo);
        assert_eq!(res.contracts.len(), 2_usize);

        Ok(())
    }
    #[test]
    fn test_query_channel_list() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // create test query data
        let to_add =
            create_channel_entry_and_string(vec![("bar", "bar1", "bar2"), ("foo", "foo1", "foo2")]);
        update_channels(deps.as_mut(), to_add)?;

        // create second entry
        let to_add1 = create_channel_entry_and_string(vec![("foobar", "foobar1", "foobar2")]);
        update_channels(deps.as_mut(), to_add1)?;

        // create msgs
        // No token filter - should return up to `limit` entries
        let msg = QueryMsg::ChannelList {
            start_after: None,
            limit: Some(42_u8),
            filter: None,
        };
        let res_all = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Filter for entries after `Foo` - Alphabetically
        let msg = QueryMsg::ChannelList {
            start_after: Some(ChannelEntry {
                connected_chain: "foo".to_string(),
                protocol: "foo1".to_string(),
            }),
            limit: Some(42_u8),
            filter: None,
        };
        let res_foobar = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Return first entry - Alphabetically
        let msg = QueryMsg::ChannelList {
            start_after: None,
            limit: Some(1_u8),
            filter: None,
        };
        let res_bar = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // Stage data for equality test

        // Return all
        let expected_all = ChannelListResponse {
            channels: create_channel_entry_and_string(vec![
                ("bar", "bar1", "bar2"),
                ("foo", "foo1", "foo2"),
                ("foobar", "foobar1", "foobar2"),
            ]),
        };
        // Filter from `Foo`
        let expected_foobar = ChannelListResponse {
            channels: create_channel_entry_and_string(vec![("foobar", "foobar1", "foobar2")]),
        };
        // Return first entry (alphabetically)
        let expected_bar = ChannelListResponse {
            channels: create_channel_entry_and_string(vec![("bar", "bar1", "bar2")]),
        };
        // Assert
        assert_that!(&res_all).is_equal_to(expected_all);
        assert_that!(&res_foobar).is_equal_to(expected_foobar);
        assert_that!(&res_bar).is_equal_to(expected_bar);
        assert_eq!(res_all.channels.len(), 3_usize);

        Ok(())
    }

    #[test]
    fn test_query_registered_dexes() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();

        // Create test data
        let to_add: Vec<String> = vec!["foo".to_string(), "bar".to_string()];
        update_registered_dexes(deps.as_mut(), to_add)?;

        // create duplicate entry
        let to_add1: Vec<String> = vec!["foo".to_string(), "foo".to_string()];
        update_registered_dexes(deps.as_mut(), to_add1)?;

        // create msg
        let msg = QueryMsg::RegisteredDexes {};
        // deserialize response
        let res: RegisteredDexesResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;

        // comparisons
        let expected = RegisteredDexesResponse {
            dexes: vec!["foo".to_string(), "bar".to_string()],
        };
        // tests
        assert_that!(&res).is_equal_to(expected);
        // assert no duplication
        assert!(res.dexes.len() == 2_usize);
        assert!(res.dexes[0] == ("foo"));
        assert!(res.dexes[1] == ("bar"));
        Ok(())
    }
    #[test]
    fn test_query_pools() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;

        // create DexAssetPairing
        update_asset_pairing("btc", "eth", "foo", 42, deps.as_mut(), api)?;

        // create msg
        let msg = QueryMsg::Pools {
            pairings: vec![create_dex_asset_pairing("btc", "eth", "foo")],
        };
        let res: PoolsResponse = from_binary(&query_helper(deps.as_ref(), msg)?)?;
        //comparisons
        let expected = ASSET_PAIRINGS
            .load(
                &deps.storage,
                &create_dex_asset_pairing("btc", "eth", "foo"),
            )
            .unwrap();
        let expected = PoolsResponse {
            pools: vec![(create_dex_asset_pairing("btc", "eth", "foo"), expected)],
        };
        // assert
        println!("{res:?}");
        assert_eq!(&res, &expected);
        Ok(())
    }

    #[test]
    fn test_query_pool_list() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        let api = deps.api;

        // create first pool entry
        update_asset_pairing("btc", "eth", "bar", 42, deps.as_mut(), api)?;

        // create second pool entry
        update_asset_pairing("juno", "atom", "foo", 69, deps.as_mut(), api)?;

        // create duplicate pool entry
        update_asset_pairing("juno", "atom", "foo", 69, deps.as_mut(), api)?;

        // create msgs bar/ foo / foo using `start_after` as filter
        let msg_bar = create_pool_list_msg(
            Some(create_asset_pairing_filter("btc", "eth", None)?),
            None,
            None,
        )?;
        let res_bar: PoolsResponse = from_binary(&query_helper(deps.as_ref(), msg_bar)?)?;

        let msg_foo = create_pool_list_msg(
            Some(create_asset_pairing_filter("juno", "atom", None)?),
            None,
            Some(42),
        )?;
        let res_foo: PoolsResponse = from_binary(&query_helper(deps.as_ref(), msg_foo)?)?;

        let msg_foo_using_start_after = create_pool_list_msg(
            Some(AssetPairingFilter {
                asset_pair: None,
                dex: None,
            }),
            Some(create_dex_asset_pairing("btc", "eth", "bar")),
            Some(42),
        )?;
        let res_foo_using_start_after: PoolsResponse =
            from_binary(&query_helper(deps.as_ref(), msg_foo_using_start_after)?)?;

        // create comparisons - bar / foo / all
        let expected_bar =
            load_asset_pairing_into_pools_response("btc", "eth", "bar", deps.as_mut())?;
        let expected_foo =
            load_asset_pairing_into_pools_response("juno", "atom", "foo", deps.as_mut())?;
        let expected_all_bar = ASSET_PAIRINGS
            .load(
                &deps.storage,
                &create_dex_asset_pairing("btc", "eth", "bar"),
            )
            .unwrap();
        let expected_all_foo = ASSET_PAIRINGS
            .load(
                &deps.storage,
                &create_dex_asset_pairing("juno", "atom", "foo"),
            )
            .unwrap();
        let expected_all = PoolsResponse {
            pools: vec![
                (
                    create_dex_asset_pairing("btc", "eth", "bar"),
                    expected_all_bar,
                ),
                (
                    create_dex_asset_pairing("juno", "atom", "foo"),
                    expected_all_foo,
                ),
            ],
        };
        // comparison all
        let msg_all = create_pool_list_msg(None, None, Some(42))?;
        let res_all: PoolsResponse = from_binary(&query_helper(deps.as_ref(), msg_all)?)?;

        // assert
        assert_eq!(&res_bar, &expected_bar);
        assert_eq!(&res_foo, &expected_foo);
        assert!(res_foo.pools.len() == 1usize);
        assert_eq!(&res_foo_using_start_after, &expected_foo);
        assert_eq!(&res_all, &expected_all);
        Ok(())
    }
    #[test]
    fn test_query_pool_metadata() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        // create metadata entries
        let bar_key = UniquePoolId::new(42);
        let bar_metadata = create_pool_metadata("bar", "btc", "eth");
        let insert_bar = |_| -> StdResult<PoolMetadata> { Ok(bar_metadata) };
        POOL_METADATA.update(&mut deps.storage, bar_key, insert_bar)?;

        let foo_key = UniquePoolId::new(69);
        let foo_metadata = create_pool_metadata("foo", "juno", "atom");
        let insert_foo = |_| -> StdResult<PoolMetadata> { Ok(foo_metadata) };
        POOL_METADATA.update(&mut deps.storage, foo_key, insert_foo)?;

        // create msgs
        let msg_bar = QueryMsg::PoolMetadatas {
            ids: vec![UniquePoolId::new(42)],
        };
        let res_bar: PoolMetadatasResponse = from_binary(&query_helper(deps.as_ref(), msg_bar)?)?;

        let msg_foo = QueryMsg::PoolMetadatas {
            ids: vec![UniquePoolId::new(69)],
        };
        let res_foo: PoolMetadatasResponse = from_binary(&query_helper(deps.as_ref(), msg_foo)?)?;

        // create comparisons
        let expected_bar = PoolMetadatasResponse {
            metadatas: vec![(
                UniquePoolId::new(42),
                PoolMetadata::new(
                    "bar",
                    PoolType::Stable,
                    vec!["btc".to_string(), "eth".to_string()],
                ),
            )],
        };
        let expected_foo = PoolMetadatasResponse {
            metadatas: vec![(
                UniquePoolId::new(69),
                PoolMetadata::new(
                    "foo",
                    PoolType::Stable,
                    vec!["juno".to_string(), "atom".to_string()],
                ),
            )],
        };
        assert_eq!(&res_bar, &expected_bar);
        println!("res_foo:{res_foo:?} expected_foo:{expected_foo:?}");
        assert_eq!(&res_foo, &expected_foo);

        Ok(())
    }

    #[test]
    fn test_query_pool_metadata_list() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut()).unwrap();
        // create metadata entries
        let bar_key = UniquePoolId::new(42);
        let bar_metadata = create_pool_metadata("bar", "btc", "eth");
        let insert_bar = |_| -> StdResult<PoolMetadata> { Ok(bar_metadata.clone()) };
        POOL_METADATA.update(&mut deps.storage, bar_key, insert_bar)?;

        let msg_bar = QueryMsg::PoolMetadataList {
            filter: Some(PoolMetadataFilter {
                pool_type: Some(PoolType::Stable),
            }),
            start_after: None,
            limit: None,
        };
        let res_bar: PoolMetadatasResponse = from_binary(&query_helper(deps.as_ref(), msg_bar)?)?;
        let expected_bar = PoolMetadatasResponse {
            metadatas: vec![(bar_key, bar_metadata.clone())],
        };
        assert_that!(res_bar).is_equal_to(expected_bar);

        let foo_key = UniquePoolId::new(69);
        let foo_metadata = create_pool_metadata("foo", "juno", "atom");
        let insert_foo = |_| -> StdResult<PoolMetadata> { Ok(foo_metadata.clone()) };
        POOL_METADATA.update(&mut deps.storage, foo_key, insert_foo)?;

        let msg_both = QueryMsg::PoolMetadataList {
            filter: Some(PoolMetadataFilter {
                pool_type: Some(PoolType::Stable),
            }),
            start_after: None,
            limit: Some(42),
        };
        let res_both: PoolMetadatasResponse = from_binary(&query_helper(deps.as_ref(), msg_both)?)?;

        let expected_both = PoolMetadatasResponse {
            metadatas: vec![
                (bar_key, bar_metadata.clone()),
                (foo_key, foo_metadata.clone()),
            ],
        };
        println!("{res_both:?} {expected_both:?}");
        assert_that!(res_both).is_equal_to(expected_both);

        let msg_foo = QueryMsg::PoolMetadataList {
            filter: Some(PoolMetadataFilter {
                pool_type: Some(PoolType::Stable),
            }),
            start_after: Some(bar_key),
            limit: Some(42),
        };
        let res_foo: PoolMetadatasResponse = from_binary(&query_helper(deps.as_ref(), msg_foo)?)?;

        let expected_foo = PoolMetadatasResponse {
            metadatas: vec![(foo_key, foo_metadata)],
        };

        assert_that!(res_foo).is_equal_to(expected_foo);
        Ok(())
    }
}
