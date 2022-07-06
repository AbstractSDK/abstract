use abstract_sdk::memory::{query_assets_from_mem, query_contracts_from_mem};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult};

use abstract_os::memory::{
    state::{ASSET_ADDRESSES, CONTRACT_ADDRESSES},
    QueryAssetListResponse, QueryAssetsResponse, QueryContractListResponse, QueryContractsResponse,
};
use cw_asset::AssetInfo;
use cw_storage_plus::Bound;

const DEFAULT_LIMIT: u8 = 15;
const MAX_LIMIT: u8 = 25;

pub fn query_assets(deps: Deps, env: Env, asset_names: Vec<String>) -> StdResult<Binary> {
    let assets = query_assets_from_mem(deps, &env.contract.address, &asset_names)?;
    let vector = assets.into_iter().map(|(v, k)| (v, k)).collect();
    to_binary(&QueryAssetsResponse { assets: vector })
}

pub fn query_contract(deps: Deps, env: Env, names: Vec<String>) -> StdResult<Binary> {
    let contracts = query_contracts_from_mem(deps, &env.contract.address, &names)?;
    let vector = contracts
        .into_iter()
        .map(|(v, k)| (v, k.to_string()))
        .collect();
    to_binary(&QueryContractsResponse { contracts: vector })
}

pub fn query_asset_list(
    deps: Deps,
    last_asset_name: Option<String>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_asset_name.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(String, AssetInfo)>, _> = ASSET_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Descending)
        .take(limit)
        .collect();

    to_binary(&QueryAssetListResponse { assets: res? })
}

pub fn query_contract_list(
    deps: Deps,
    last_contract_name: Option<String>,
    limit: Option<u8>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_contract_name.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(String, Addr)>, _> = CONTRACT_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Descending)
        .take(limit)
        .collect();
    to_binary(&QueryContractListResponse {
        contracts: res?.into_iter().map(|(x, a)| (x, a.to_string())).collect(),
    })
}
