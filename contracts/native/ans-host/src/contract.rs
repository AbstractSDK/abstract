use crate::commands::*;
use crate::error::AnsHostError;
use crate::queries;
use abstract_os::ans_host::state::{
    Config, ADMIN, ASSET_ADDRESSES, CONFIG, REGISTERED_DEXES, REV_ASSET_ADDRESSES,
};
use abstract_os::ans_host::{AssetMapEntry, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

pub type AnsHostResult = Result<Response, AnsHostError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use abstract_os::ANS_HOST;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> AnsHostResult {
    set_contract_version(deps.storage, ANS_HOST, CONTRACT_VERSION)?;

    // Initialize the config
    CONFIG.save(
        deps.storage,
        &Config {
            next_unique_pool_id: 1.into(),
        },
    )?;

    // Initialize the dexes
    REGISTERED_DEXES.save(deps.storage, &vec![])?;

    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> AnsHostResult {
    handle_message(deps, info, env, msg)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => queries::query_config(deps),
        QueryMsg::Assets { names } => queries::query_assets(deps, env, names),
        QueryMsg::AssetList {
            page_token,
            page_size,
        } => queries::query_asset_list(deps, page_token, page_size),
        QueryMsg::AssetInfos { infos } => queries::query_asset_infos(deps, env, infos),
        QueryMsg::AssetInfoList {
            page_token,
            page_size,
        } => queries::query_asset_info_list(deps, page_token, page_size),
        QueryMsg::Contracts { names } => queries::query_contract(deps, env, names),
        QueryMsg::ContractList {
            page_token,
            page_size,
        } => queries::query_contract_list(deps, page_token, page_size),
        QueryMsg::Channels { names } => queries::query_channels(deps, env, names),
        QueryMsg::ChannelList {
            page_token,
            page_size,
        } => queries::query_channel_list(deps, page_token, page_size),
        QueryMsg::RegisteredDexes {} => queries::query_registered_dexes(deps, env),
        QueryMsg::PoolList {
            filter,
            page_token,
            page_size,
        } => queries::list_pool_entries(deps, filter, page_token, page_size),

        QueryMsg::Pools { keys } => queries::query_pool_entries(deps, keys),
        QueryMsg::PoolMetadatas { keys } => queries::query_pool_metadatas(deps, keys),
        QueryMsg::PoolMetadataList {
            filter,
            page_token,
            page_size,
        } => queries::list_pool_metadata_entries(deps, filter, page_token, page_size),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
    if storage_version < version {
        set_contract_version(deps.storage, ANS_HOST, CONTRACT_VERSION)?;
    }

    let res: Result<Vec<AssetMapEntry>, _> = ASSET_ADDRESSES
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    for (info, asset) in res? {
        REV_ASSET_ADDRESSES.save(deps.storage, asset, &info)?;
    }
    Ok(Response::default())
}
