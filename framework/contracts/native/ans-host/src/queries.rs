use abstract_std::{
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
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdError, StdResult, Storage};
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

    to_json_binary(&res)
}

pub fn query_assets(deps: Deps, _env: Env, keys: Vec<String>) -> StdResult<Binary> {
    let assets = keys
        .into_iter()
        .map(|name| {
            let key = AssetEntry::new(&name);
            let value = ASSET_ADDRESSES.load(deps.storage, &key)?;
            Ok((key, value))
        })
        .collect::<StdResult<_>>()?;

    to_json_binary(&AssetsResponse { assets })
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

    to_json_binary(&AssetListResponse { assets: res? })
}

pub fn query_asset_infos(
    deps: Deps,
    _env: Env,
    keys: Vec<AssetInfoUnchecked>,
) -> StdResult<Binary> {
    let infos = keys
        .into_iter()
        .map(|info| {
            let key = info
                .check(deps.api, None)
                .map_err(|err| StdError::generic_err(err.to_string()))?;
            let value = REV_ASSET_ADDRESSES.load(deps.storage, &key)?;
            Ok((key, value))
        })
        .collect::<StdResult<_>>()?;

    to_json_binary(&AssetInfosResponse { infos })
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

    to_json_binary(&AssetInfoListResponse { infos: res? })
}

pub fn query_contract(deps: Deps, _env: Env, keys: Vec<ContractEntry>) -> StdResult<Binary> {
    let contracts = keys
        .into_iter()
        .map(|key| {
            let value = CONTRACT_ADDRESSES.load(deps.storage, &key)?;
            Ok((key, value))
        })
        .collect::<StdResult<_>>()?;

    to_json_binary(&ContractsResponse { contracts })
}

pub fn query_channels(deps: Deps, _env: Env, keys: Vec<ChannelEntry>) -> StdResult<Binary> {
    let channels = keys
        .into_iter()
        .map(|key| {
            let value = CHANNELS.load(deps.storage, &key)?;
            Ok((key, value))
        })
        .collect::<StdResult<_>>()?;

    to_json_binary(&ChannelsResponse { channels })
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

    to_json_binary(&ContractListResponse { contracts: res? })
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

    to_json_binary(&ChannelListResponse { channels: res? })
}

pub fn query_registered_dexes(deps: Deps, _env: Env) -> StdResult<Binary> {
    let dexes = REGISTERED_DEXES.load(deps.storage)?;

    to_json_binary(&RegisteredDexesResponse { dexes })
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

    let entry_list = match (asset_pair_filter, dex_filter) {
        (Some((asset_x, asset_y)), Some(dex_filter)) => {
            // We have the full key, so load the entry
            let key = DexAssetPairing::new(asset_x, asset_y, &dex_filter);
            let entry = load_asset_pairing_entry(deps.storage, key)?;
            vec![entry]
        }
        (Some((asset_x, asset_y)), None) => {
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
        }
        (None, dex_filter) => {
            let start_bound: Option<Bound<&DexAssetPairing>> =
                start_after.as_ref().map(Bound::exclusive);

            // We have no filter, so load all the entries
            ASSET_PAIRINGS
                .range(deps.storage, start_bound, None, Order::Ascending)
                .filter(|e| {
                    dex_filter
                        .as_ref()
                        .map_or(true, |f| f == e.as_ref().unwrap().0.dex())
                })
                .collect::<StdResult<_>>()?
        }
    };

    to_json_binary(&PoolAddressListResponse { pools: entry_list })
}

/// Query the pool ids based on the actual keys
pub fn query_pool_entries(deps: Deps, keys: Vec<DexAssetPairing>) -> StdResult<Binary> {
    let mut entries: Vec<AssetPairingMapEntry> = vec![];
    for key in keys.into_iter() {
        let entry = load_asset_pairing_entry(deps.storage, key)?;

        entries.push(entry);
    }

    to_json_binary(&PoolsResponse { pools: entries })
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

    to_json_binary(&PoolMetadatasResponse { metadatas: entries })
}

pub fn list_pool_metadata_entries(
    deps: Deps,
    filter: Option<PoolMetadataFilter>,
    start_after: Option<UniquePoolId>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = start_after.map(Bound::exclusive);

    let PoolMetadataFilter {
        pool_type: pool_type_filter,
    } = filter.unwrap_or_default();

    let res: Result<Vec<(UniquePoolId, PoolMetadata)>, _> = POOL_METADATA
        // If the asset_pair_filter is provided, we must use that prefix...
        .range(deps.storage, start_bound, None, Order::Ascending)
        .filter(|e| {
            let pool_type = &e.as_ref().unwrap().1.pool_type;
            pool_type_filter.as_ref().map_or(true, |f| f == pool_type)
        })
        .take(limit)
        .collect();

    to_json_binary(&PoolMetadataListResponse { metadatas: res? })
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
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use crate::{
        contract,
        contract::{instantiate, AnsHostResult},
        error::AnsHostError,
    };
    use abstract_std::{
        ans_host::*,
        objects::{pool_id::PoolAddressBase, PoolType, TruncatedChainId},
    };
    use abstract_testing::{addresses::AbstractMockAddrs, mock_env_validated};
    use cosmwasm_std::{from_json, testing::*, Addr, DepsMut, OwnedDeps};
    use cw_asset::AssetInfo;
    use speculoos::prelude::*;
    use std::str::FromStr;

    type AnsHostTestResult = Result<(), AnsHostError>;

    fn mock_init(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) -> AnsHostResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);
        let admin = info.sender.to_string();
        let env = mock_env_validated(deps.api);

        instantiate(deps.as_mut(), env, info, InstantiateMsg { admin })
    }

    fn query_helper(
        deps: &OwnedDeps<MockStorage, MockApi, MockQuerier>,
        msg: QueryMsg,
    ) -> StdResult<Binary> {
        let env = mock_env_validated(deps.api as MockApi);

        let res = contract::query(deps.as_ref(), env, msg)?;
        Ok(res)
    }

    fn query_asset_list_msg(token: String, size: usize) -> QueryMsg {
        QueryMsg::AssetList {
            start_after: (Some(token)),
            limit: (Some(size as u8)),
            filter: None,
        }
    }

    fn create_test_assets(input: Vec<(&str, &Addr)>, api: MockApi) -> Vec<(String, AssetInfo)> {
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
        input: Vec<(&str, &str, &Addr)>,
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
                        connected_chain: TruncatedChainId::from_string(
                            input.0.to_string().to_ascii_lowercase(),
                        )
                        .unwrap(),
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
                connected_chain: TruncatedChainId::from_string(
                    input.0.to_string().to_ascii_lowercase(),
                )
                .unwrap(),
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
            pool_address: PoolAddressBase::contract(api.addr_make(pool_id))
                .check(&api)
                .unwrap(),
        }])
    }

    fn create_pool_metadata(dex: &str, asset_x: &str, asset_y: &str) -> PoolMetadata {
        PoolMetadata::new(
            dex,
            PoolType::Stable,
            vec![asset_x.to_string(), asset_y.to_string()],
        )
    }

    #[coverage_helper::test]
    fn test_query_assets() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();
        let api = deps.api;

        // create test query data
        let test_assets = create_test_assets(
            vec![
                ("bar", &api.addr_make("bar")),
                ("foo", &api.addr_make("foo")),
            ],
            api,
        );
        update_asset_addresses(deps.as_mut(), test_assets)?;
        // create msg
        let msg = QueryMsg::Assets {
            names: vec!["bar".to_string(), "foo".to_string()],
        };
        // send query message
        let res: AssetsResponse = from_json(query_helper(&deps, msg)?)?;

        // Stage data for equality test
        let expected = create_asset_response(create_test_assets(
            vec![
                ("bar", &deps.api.addr_make("bar")),
                ("foo", &deps.api.addr_make("foo")),
            ],
            api,
        ));
        // Assert
        assert_that!(&res).is_equal_to(&expected);

        Ok(())
    }

    #[coverage_helper::test]
    fn test_query_contract() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();

        // create test query data
        let to_add =
            create_contract_entry_and_string(vec![("foo", "foo", &deps.api.addr_make("foo"))]);
        update_contract_addresses(deps.as_mut(), to_add)?;
        // create, send and deserialise msg
        let msg = QueryMsg::Contracts {
            entries: create_contract_entry(vec![("foo", "foo")]),
        };
        let res: ContractsResponse = from_json(query_helper(&deps, msg)?)?;

        // Stage data for equality test
        let expected = ContractsResponse {
            contracts: create_contract_entry_and_string(vec![(
                "foo",
                "foo",
                &deps.api.addr_make("foo"),
            )])
            .into_iter()
            .map(|(a, b)| (a, Addr::unchecked(b)))
            .collect(),
        };

        // Assert
        assert_that!(&res).is_equal_to(&expected);

        Ok(())
    }

    #[coverage_helper::test]
    fn test_query_channels() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();

        // create test query data
        let to_add = create_channel_entry_and_string(vec![("foo", "foo", "foo")]);
        update_channels(deps.as_mut(), to_add)?;
        // create duplicate entry
        let to_add1 = create_channel_entry_and_string(vec![("foo", "foo", "foo")]);
        update_channels(deps.as_mut(), to_add1)?;

        // create and send and deserialise msg
        let msg = create_channel_msg(vec![("foo", "foo")]);
        let res: ChannelsResponse = from_json(query_helper(&deps, msg)?)?;

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

    #[coverage_helper::test]
    fn test_query_asset_list() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();
        let api = deps.api;

        // create test query data
        let test_assets = create_test_assets(
            vec![
                ("foo", &deps.api.addr_make("foo")),
                ("bar", &deps.api.addr_make("bar")),
            ],
            api,
        );
        update_asset_addresses(deps.as_mut(), test_assets)?;

        // create second entry
        let test_assets1 = create_test_assets(vec![("foobar", &deps.api.addr_make("foobar"))], api);
        update_asset_addresses(deps.as_mut(), test_assets1)?;

        // create duplicate entry
        let test_assets_duplicate =
            create_test_assets(vec![("foobar", &deps.api.addr_make("foobar"))], api);
        update_asset_addresses(deps.as_mut(), test_assets_duplicate)?;

        // return all entries
        let msg = query_asset_list_msg("".to_string(), 42);
        let res: AssetListResponse = from_json(query_helper(&deps, msg)?)?;

        // limit response to 1st result - entries are stored alphabetically
        let msg = query_asset_list_msg("".to_string(), 1);
        let res_first_entry: AssetListResponse = from_json(query_helper(&deps, msg)?)?;

        // results after specified entry
        let msg = query_asset_list_msg("foo".to_string(), 1);
        let res_of_foobar: AssetListResponse = from_json(query_helper(&deps, msg)?)?;

        // Stage data for equality test
        let expected = create_asset_list_response(create_test_assets(
            vec![
                ("bar", &deps.api.addr_make("bar")),
                ("foo", &deps.api.addr_make("foo")),
                ("foobar", &deps.api.addr_make("foobar")),
            ],
            api,
        ));

        let expected_foobar = create_asset_list_response(create_test_assets(
            vec![("foobar", &deps.api.addr_make("foobar"))],
            api,
        ));
        let expected_bar = create_asset_list_response(create_test_assets(
            vec![("bar", &deps.api.addr_make("bar"))],
            api,
        ));

        assert_that!(res).is_equal_to(&expected);
        assert_that!(res_first_entry).is_equal_to(&expected_bar);
        assert_that!(&res_of_foobar).is_equal_to(&expected_foobar);

        Ok(())
    }
    #[coverage_helper::test]
    fn test_query_asset_list_above_max() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();
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
        let res: AssetListResponse = from_json(query_helper(&deps, msg)?)?;
        assert!(res.assets.len() == 25_usize);

        // Assert that despite 30 entries the returned data is capped at the `MAX_LIMIT` of 25 results
        assert!(res.assets.len() == 25_usize);
        Ok(())
    }
    #[coverage_helper::test]
    fn test_query_contract_list() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();

        // create test query data
        let to_add =
            create_contract_entry_and_string(vec![("foo", "foo1", &deps.api.addr_make("foo2"))]);
        update_contract_addresses(deps.as_mut(), to_add)?;

        // create second entry
        let to_add1 =
            create_contract_entry_and_string(vec![("bar", "bar1", &deps.api.addr_make("bar2"))]);
        update_contract_addresses(deps.as_mut(), to_add1)?;

        // create duplicate entry
        let to_add1 =
            create_contract_entry_and_string(vec![("bar", "bar1", &deps.api.addr_make("bar2"))]);
        update_contract_addresses(deps.as_mut(), to_add1)?;

        // create msgs
        let msg = QueryMsg::ContractList {
            start_after: None,
            limit: Some(42_u8),
            filter: None,
        };
        let res: ContractListResponse = from_json(query_helper(&deps, msg)?)?;

        let msg = QueryMsg::ContractList {
            start_after: Some(ContractEntry {
                protocol: "bar".to_string().to_ascii_lowercase(),
                contract: "bar1".to_string().to_ascii_lowercase(),
            }),
            limit: Some(42_u8),
            filter: None,
        };
        let res_expect_foo: ContractListResponse = from_json(query_helper(&deps, msg)?)?;

        // Stage data for equality test
        let expected = ContractListResponse {
            contracts: create_contract_entry_and_string(vec![
                ("bar", "bar1", &deps.api.addr_make("bar2")),
                ("foo", "foo1", &deps.api.addr_make("foo2")),
            ])
            .into_iter()
            .map(|(a, b)| (a, Addr::unchecked(b)))
            .collect(),
        };

        let expected_foo = ContractListResponse {
            contracts: create_contract_entry_and_string(vec![(
                "foo",
                "foo1",
                &deps.api.addr_make("foo2"),
            )])
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
    #[coverage_helper::test]
    fn test_query_channel_list() -> AnsHostTestResult {
        // arrange mocks
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();

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
        let res_all = from_json(query_helper(&deps, msg)?)?;

        // Filter for entries after `Foo` - Alphabetically
        let msg = QueryMsg::ChannelList {
            start_after: Some(ChannelEntry {
                connected_chain: TruncatedChainId::from_str("foo").unwrap(),
                protocol: "foo1".to_string(),
            }),
            limit: Some(42_u8),
            filter: None,
        };
        let res_foobar = from_json(query_helper(&deps, msg)?)?;

        // Return first entry - Alphabetically
        let msg = QueryMsg::ChannelList {
            start_after: None,
            limit: Some(1_u8),
            filter: None,
        };
        let res_bar = from_json(query_helper(&deps, msg)?)?;

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

    #[coverage_helper::test]
    fn test_query_registered_dexes() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();

        // Create test data
        let to_add: Vec<String> = vec!["foo".to_string(), "bar".to_string()];
        update_registered_dexes(deps.as_mut(), to_add)?;

        // create duplicate entry
        let to_add1: Vec<String> = vec!["foo".to_string(), "foo".to_string()];
        update_registered_dexes(deps.as_mut(), to_add1)?;

        // create msg
        let msg = QueryMsg::RegisteredDexes {};
        // deserialize response
        let res: RegisteredDexesResponse = from_json(query_helper(&deps, msg)?)?;

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
    #[coverage_helper::test]
    fn test_query_pools() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();
        let api = deps.api;

        // create DexAssetPairing
        update_asset_pairing("btc", "eth", "foo", 42, deps.as_mut(), api)?;

        // create msg
        let msg = QueryMsg::Pools {
            pairings: vec![create_dex_asset_pairing("btc", "eth", "foo")],
        };
        let res: PoolsResponse = from_json(query_helper(&deps, msg)?)?;
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

    #[coverage_helper::test]
    fn test_query_pool_list() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();
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
        let res_bar: PoolsResponse = from_json(query_helper(&deps, msg_bar)?)?;

        // Exact filter
        let msg_full_filter_bar = create_pool_list_msg(
            Some(create_asset_pairing_filter(
                "btc",
                "eth",
                Some("bar".to_string()),
            )?),
            None,
            None,
        )?;
        let res_full_filter_bar: PoolsResponse =
            from_json(query_helper(&deps, msg_full_filter_bar)?)?;

        let msg_foo = create_pool_list_msg(
            Some(create_asset_pairing_filter("juno", "atom", None)?),
            None,
            Some(42),
        )?;
        let res_foo: PoolsResponse = from_json(query_helper(&deps, msg_foo)?)?;

        let msg_foo_using_start_after = create_pool_list_msg(
            Some(AssetPairingFilter {
                asset_pair: None,
                dex: None,
            }),
            Some(create_dex_asset_pairing("btc", "eth", "bar")),
            Some(42),
        )?;
        let res_foo_using_start_after: PoolsResponse =
            from_json(query_helper(&deps, msg_foo_using_start_after)?)?;

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
        let res_all: PoolsResponse = from_json(query_helper(&deps, msg_all)?)?;

        // assert
        assert_eq!(&res_bar, &expected_bar);
        assert_eq!(&res_full_filter_bar, &expected_bar);
        assert_eq!(&res_foo, &expected_foo);
        assert!(res_foo.pools.len() == 1usize);
        assert_eq!(&res_foo_using_start_after, &expected_foo);
        assert_eq!(&res_all, &expected_all);
        Ok(())
    }
    #[coverage_helper::test]
    fn test_query_pool_metadata() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();
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
        let res_bar: PoolMetadatasResponse = from_json(query_helper(&deps, msg_bar)?)?;

        let msg_foo = QueryMsg::PoolMetadatas {
            ids: vec![UniquePoolId::new(69)],
        };
        let res_foo: PoolMetadatasResponse = from_json(query_helper(&deps, msg_foo)?)?;

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

    #[coverage_helper::test]
    fn test_query_pool_metadata_list() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();
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
        let res_bar: PoolMetadatasResponse = from_json(query_helper(&deps, msg_bar)?)?;
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
        let res_both: PoolMetadatasResponse = from_json(query_helper(&deps, msg_both)?)?;

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
        let res_foo: PoolMetadatasResponse = from_json(query_helper(&deps, msg_foo)?)?;

        let expected_foo = PoolMetadatasResponse {
            metadatas: vec![(foo_key, foo_metadata)],
        };

        assert_that!(res_foo).is_equal_to(expected_foo);
        Ok(())
    }

    #[coverage_helper::test]
    fn test_query_asset_infos() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();
        let native_1 = AssetInfo::native("foo");
        let native_2 = AssetInfo::native("bar");
        let cw20_1 = AssetInfo::cw20(deps.api.addr_make("foo"));
        let cw20_2 = AssetInfo::cw20(deps.api.addr_make("bar"));

        // create asset entries
        REV_ASSET_ADDRESSES.save(&mut deps.storage, &native_1, &AssetEntry::new("foo_n"))?;
        REV_ASSET_ADDRESSES.save(&mut deps.storage, &native_2, &AssetEntry::new("bar_n"))?;
        REV_ASSET_ADDRESSES.save(&mut deps.storage, &cw20_1, &AssetEntry::new("foo_ft"))?;
        REV_ASSET_ADDRESSES.save(&mut deps.storage, &cw20_2, &AssetEntry::new("bar_ft"))?;

        let msg = QueryMsg::AssetInfos {
            infos: vec![
                native_1.clone().into(),
                native_2.clone().into(),
                cw20_1.clone().into(),
                cw20_2.clone().into(),
            ],
        };
        let res: AssetInfosResponse = from_json(query_helper(&deps, msg)?)?;
        let expected_bar = AssetInfosResponse {
            infos: vec![
                (native_1, AssetEntry::new("foo_n")),
                (native_2, AssetEntry::new("bar_n")),
                (cw20_1, AssetEntry::new("foo_ft")),
                (cw20_2, AssetEntry::new("bar_ft")),
            ],
        };
        assert_eq!(res, expected_bar);

        // Query invalid asset
        let res = query_helper(
            &deps,
            QueryMsg::AssetInfos {
                infos: vec![AssetInfoUnchecked::cw20("invalid_addr".to_string())],
            },
        );
        assert!(res.is_err());
        // Query not saved asset
        let res = query_helper(
            &deps,
            QueryMsg::AssetInfos {
                infos: vec![AssetInfoUnchecked::native("not_saved".to_string())],
            },
        );
        assert!(res.is_err());
        Ok(())
    }

    #[coverage_helper::test]
    fn test_query_asset_infos_list() -> AnsHostTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps).unwrap();
        let native_1 = AssetInfo::native("foo");
        let native_2 = AssetInfo::native("bar");
        let cw20_1 = AssetInfo::cw20(deps.api.addr_make("foo"));
        let cw20_2 = AssetInfo::cw20(deps.api.addr_make("bar"));

        // create asset entries
        REV_ASSET_ADDRESSES.save(&mut deps.storage, &native_1, &AssetEntry::new("foo_n"))?;
        REV_ASSET_ADDRESSES.save(&mut deps.storage, &native_2, &AssetEntry::new("bar_n"))?;
        REV_ASSET_ADDRESSES.save(&mut deps.storage, &cw20_1, &AssetEntry::new("foo_ft"))?;
        REV_ASSET_ADDRESSES.save(&mut deps.storage, &cw20_2, &AssetEntry::new("bar_ft"))?;

        let msg = QueryMsg::AssetInfoList {
            filter: None,
            start_after: None,
            limit: None,
        };
        let res: AssetInfoListResponse = from_json(query_helper(&deps, msg)?)?;
        let expected_infos = AssetInfoListResponse {
            infos: vec![
                (cw20_1.clone(), AssetEntry::new("foo_ft")),
                (cw20_2.clone(), AssetEntry::new("bar_ft")),
                (native_2.clone(), AssetEntry::new("bar_n")),
                (native_1.clone(), AssetEntry::new("foo_n")),
            ],
        };
        assert_eq!(res, expected_infos);

        // Start after
        let msg = QueryMsg::AssetInfoList {
            filter: None,
            start_after: Some(cw20_2.clone().into()),
            limit: None,
        };
        let res: AssetInfoListResponse = from_json(query_helper(&deps, msg)?)?;
        let expected_infos = AssetInfoListResponse {
            infos: vec![
                (native_2, AssetEntry::new("bar_n")),
                (native_1, AssetEntry::new("foo_n")),
            ],
        };
        assert_eq!(res, expected_infos);

        // Limit
        let msg = QueryMsg::AssetInfoList {
            filter: None,
            start_after: None,
            limit: Some(1),
        };
        let res: AssetInfoListResponse = from_json(query_helper(&deps, msg)?)?;
        let expected_infos = AssetInfoListResponse {
            infos: vec![(cw20_1.clone(), AssetEntry::new("foo_ft"))],
        };
        assert_eq!(res, expected_infos);

        Ok(())
    }
}
