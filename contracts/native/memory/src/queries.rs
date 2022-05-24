use abstract_os::native::memory::queries::{query_assets_from_mem, query_contracts_from_mem};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

use abstract_os::native::memory::msg::{AssetQueryResponse, ContractQueryResponse};

pub fn query_assets(deps: Deps, env: Env, asset_names: Vec<String>) -> StdResult<Binary> {
    let assets = query_assets_from_mem(deps, &env.contract.address, &asset_names)?;
    let vector = assets.into_iter().map(|(v, k)| (v, k)).collect();
    to_binary(&AssetQueryResponse { assets: vector })
}

pub fn query_contract(deps: Deps, env: Env, names: Vec<String>) -> StdResult<Binary> {
    let contracts = query_contracts_from_mem(deps, &env.contract.address, &names)?;
    let vector = contracts
        .into_iter()
        .map(|(v, k)| (v, k.to_string()))
        .collect();
    to_binary(&ContractQueryResponse { contracts: vector })
}
