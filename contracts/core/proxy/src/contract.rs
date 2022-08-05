use std::collections::HashSet;
use std::convert::TryInto;

use abstract_os::objects::core::OS_ID;
use abstract_os::objects::{AssetEntry, UncheckedContractEntry};
use abstract_sdk::memory::Memory;
use abstract_sdk::Resolve;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response,
    StdError, StdResult, Uint128,
};
use cw_storage_plus::Bound;

use crate::error::ProxyError;
use abstract_os::objects::proxy_asset::{
    get_pair_asset_names, other_asset_name, ProxyAsset, UncheckedProxyAsset, ValueRef,
};
use abstract_os::proxy::state::{State, ADMIN, MEMORY, STATE, VAULT_ASSETS};
use abstract_os::proxy::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryConfigResponse, QueryHoldingAmountResponse,
    QueryHoldingValueResponse, QueryMsg, QueryProxyAssetConfigResponse, QueryProxyAssetsResponse,
    QueryTotalValueResponse, QueryValidityResponse,
};
use abstract_os::PROXY;
use cw2::{get_contract_version, set_contract_version};
use semver::Version;
type ProxyResult = Result<Response, ProxyError>;
/*
    The proxy is the bank account of the protocol. It owns the liquidity and acts as a proxy contract.
    Whitelisted dApps construct messages for this contract. The dApps are controlled by Governance.
*/
// TODO: test max limit on-chain
const LIST_SIZE_LIMIT: usize = 15;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_LIMIT: u8 = 5;
const MAX_LIMIT: u8 = 20;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ProxyResult {
    // Use CW2 to set the contract version, this is needed for migrations
    set_contract_version(deps.storage, PROXY, CONTRACT_VERSION)?;
    OS_ID.save(deps.storage, &msg.os_id)?;
    STATE.save(deps.storage, &State { modules: vec![] })?;
    MEMORY.save(
        deps.storage,
        &Memory {
            address: deps.api.addr_validate(&msg.memory_address)?,
        },
    )?;
    let admin_addr = Some(info.sender);
    ADMIN.set(deps, admin_addr)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ProxyResult {
    match msg {
        ExecuteMsg::ModuleAction { msgs } => execute_action(deps, info, msgs),
        ExecuteMsg::SetAdmin { admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
            ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
            Ok(Response::default()
                .add_attribute("previous admin", previous_admin)
                .add_attribute("admin", admin))
        }
        ExecuteMsg::AddModule { module } => add_module(deps, info, module),
        ExecuteMsg::RemoveModule { module } => remove_module(deps, info, module),
        ExecuteMsg::UpdateAssets { to_add, to_remove } => {
            update_assets(deps, info, to_add, to_remove)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ProxyResult {
    let version: Version = CONTRACT_VERSION.parse()?;
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        set_contract_version(deps.storage, PROXY, CONTRACT_VERSION)?;
    }
    Ok(Response::default())
}

/// Executes actions forwarded by whitelisted contracts
/// This contracts acts as a proxy contract for the dApps
pub fn execute_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<CosmosMsg<Empty>>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state
        .modules
        .contains(&deps.api.addr_validate(msg_info.sender.as_str())?)
    {
        return Err(ProxyError::SenderNotWhitelisted {});
    }

    Ok(Response::new().add_messages(msgs))
}

/// Update the stored vault asset information
pub fn update_assets(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<UncheckedProxyAsset>,
    to_remove: Vec<String>,
) -> ProxyResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let memory = &MEMORY.load(deps.storage)?;
    // Check the vault size to be within the size limit to prevent running out of gas when doing lookups
    let current_vault_size = VAULT_ASSETS
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    let delta: i128 = to_add.len() as i128 - to_remove.len() as i128;
    if current_vault_size as i128 + delta > LIST_SIZE_LIMIT as i128 {
        return Err(ProxyError::AssetsLimitReached {});
    }

    for new_asset in to_add.into_iter() {
        let checked_asset = new_asset.check(deps.as_ref(), memory)?;

        VAULT_ASSETS.save(deps.storage, checked_asset.asset.clone(), &checked_asset)?;
    }

    for asset_id in to_remove {
        VAULT_ASSETS.remove(deps.storage, asset_id.into());
    }

    // Check validity of new configuration
    let validity_result = query_proxy_asset_validity(deps.as_ref())?;
    if validity_result.missing_dependencies.is_some()
        || validity_result.unresolvable_assets.is_some()
    {
        return Err(ProxyError::BadUpdate(format!("{:?}", validity_result)));
    }

    Ok(Response::new().add_attribute("action", "update_proxy_assets"))
}

/// Add a contract to the whitelist
pub fn add_module(deps: DepsMut, msg_info: MessageInfo, module: String) -> ProxyResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    if state.modules.contains(&deps.api.addr_validate(&module)?) {
        return Err(ProxyError::AlreadyInList {});
    }

    // This is a limit to prevent potentially running out of gas when doing lookups on the modules list
    if state.modules.len() >= LIST_SIZE_LIMIT {
        return Err(ProxyError::ModuleLimitReached {});
    }

    // Add contract to whitelist.
    state.modules.push(deps.api.addr_validate(&module)?);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Added contract to whitelist: ", module))
}

/// Remove a contract from the whitelist
pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module: String) -> ProxyResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    if !state.modules.contains(&deps.api.addr_validate(&module)?) {
        return Err(ProxyError::NotInList {});
    }

    // Remove contract from whitelist.
    let module_address = deps.api.addr_validate(&module)?;
    state.modules.retain(|addr| *addr != module_address);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Removed contract from whitelist: ", module))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::TotalValue {} => to_binary(&QueryTotalValueResponse {
            value: compute_total_value(deps, env)?,
        }),
        QueryMsg::HoldingAmount { identifier } => {
            let vault_asset: AssetEntry = identifier.into();
            let memory = MEMORY.load(deps.storage)?;
            let asset_info = vault_asset.resolve(deps, &memory)?;
            to_binary(&QueryHoldingAmountResponse {
                amount: asset_info.query_balance(&deps.querier, env.contract.address)?,
            })
        }
        QueryMsg::HoldingValue { identifier } => to_binary(&QueryHoldingValueResponse {
            value: compute_holding_value(deps, &env, identifier)?,
        }),
        QueryMsg::ProxyAssetConfig { identifier } => to_binary(&QueryProxyAssetConfigResponse {
            proxy_asset: VAULT_ASSETS.load(deps.storage, identifier.into())?,
        }),
        QueryMsg::ProxyAssets {
            last_asset_name,
            iter_limit,
        } => to_binary(&query_proxy_assets(deps, last_asset_name, iter_limit)?),
        QueryMsg::CheckValidity {} => to_binary(&query_proxy_asset_validity(deps)?),
    }
}

fn query_proxy_assets(
    deps: Deps,
    last_asset_name: Option<String>,
    limit: Option<u8>,
) -> StdResult<QueryProxyAssetsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = last_asset_name.as_deref().map(Bound::exclusive);

    let res: Result<Vec<(AssetEntry, ProxyAsset)>, _> = VAULT_ASSETS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect();

    let names_and_configs = res?;
    Ok(QueryProxyAssetsResponse {
        assets: names_and_configs,
    })
}

/// Returns the whitelisted modules
pub fn query_config(deps: Deps) -> StdResult<QueryConfigResponse> {
    let state = STATE.load(deps.storage)?;
    let modules: Vec<Addr> = state.modules;
    let resp = QueryConfigResponse {
        modules: modules
            .iter()
            .map(|module| -> String { module.to_string() })
            .collect(),
    };
    Ok(resp)
}

/// Returns the value of a specified asset.
pub fn compute_holding_value(deps: Deps, env: &Env, asset_entry: String) -> StdResult<Uint128> {
    let mut vault_asset: ProxyAsset = VAULT_ASSETS.load(deps.storage, asset_entry.into())?;
    let memory = MEMORY.load(deps.storage)?;
    let value = vault_asset.value(deps, env, &memory, None)?;
    Ok(value)
}

/// Computes the total value locked in this contract
pub fn compute_total_value(deps: Deps, env: Env) -> StdResult<Uint128> {
    // Get all assets from storage
    let mut all_assets = VAULT_ASSETS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(AssetEntry, ProxyAsset)>>>()?;

    let mut total_value = Uint128::zero();
    let memory = MEMORY.load(deps.storage)?;
    // Calculate their value iteratively
    for vault_asset_entry in all_assets.iter_mut() {
        total_value += vault_asset_entry.1.value(deps, &env, &memory, None)?;
    }
    Ok(total_value)
}

fn query_proxy_asset_validity(deps: Deps) -> StdResult<QueryValidityResponse> {
    // assets that resolve and have valid value-references
    let mut checked_assets: HashSet<String> = HashSet::new();
    // assets that don't resolve, they have a missing dependency
    let mut unresolvable_assets: HashSet<String> = HashSet::new();
    // assets that are missing
    let mut missing_assets: HashSet<String> = HashSet::new();
    let mut base_asset: Option<String> = None;

    let assets = VAULT_ASSETS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(AssetEntry, ProxyAsset)>>>()?;
    for (_, asset) in assets {
        resolve_asset(
            deps,
            &mut checked_assets,
            &mut unresolvable_assets,
            &mut missing_assets,
            asset,
            &mut base_asset,
        )?;
    }

    let unresolvable_assets_resp = {
        if unresolvable_assets.is_empty() {
            None
        } else {
            Some(
                unresolvable_assets
                    .into_iter()
                    .map(|asset| asset.into())
                    .collect(),
            )
        }
    };

    let missing_assets_resp = {
        if missing_assets.is_empty() {
            None
        } else {
            Some(
                missing_assets
                    .into_iter()
                    .map(|asset| asset.into())
                    .collect(),
            )
        }
    };

    Ok(QueryValidityResponse {
        unresolvable_assets: unresolvable_assets_resp,
        missing_dependencies: missing_assets_resp,
    })
}

fn resolve_asset(
    deps: Deps,
    checked_assets: &mut HashSet<String>,
    unresolvable_assets: &mut HashSet<String>,
    missing_assets: &mut HashSet<String>,
    proxy_asset: ProxyAsset,
    base: &mut Option<String>,
) -> StdResult<()> {
    let ProxyAsset {
        asset: entry,
        value_reference,
    } = proxy_asset;
    // key already checked?
    if checked_assets.contains(entry.as_str()) || unresolvable_assets.contains(entry.as_str()) {
        return Ok(());
    }

    match value_reference {
        None => {
            if base.is_some() {
                if entry.as_str() != base.as_ref().unwrap() {
                    return Err(StdError::generic_err(format!(
                        "there can only be one base asset, multiple are registered: {}, {}",
                        base.as_ref().unwrap(),
                        entry.as_str()
                    )));
                }
            } else {
                *base = Some(entry.to_string());
            }
        }
        Some(value_ref) => {
            let asset_dependencies = get_value_ref_dependencies(&value_ref, entry.to_string());
            let mut loaded_dependencies = vec![];
            for asset in asset_dependencies {
                match try_load_asset(deps, missing_assets, asset) {
                    Some(proxy_asset) => {
                        // successfully loaded dependency
                        loaded_dependencies.push(proxy_asset)
                    }
                    None => {
                        // current asset unresolvable because it has dependencies that can't be loaded.
                        unresolvable_assets.insert(entry.to_string());
                    }
                }
            }
            // proceed with dependencies that resolved and add entry as checked
            checked_assets.insert(entry.to_string());
            for dep in loaded_dependencies {
                resolve_asset(
                    deps,
                    checked_assets,
                    unresolvable_assets,
                    missing_assets,
                    dep,
                    base,
                )?
            }
        }
    }
    Ok(())
}

fn try_load_asset(
    deps: Deps,
    missing_assets: &mut HashSet<String>,
    key: AssetEntry,
) -> Option<ProxyAsset> {
    let maybe_proxy_asset = VAULT_ASSETS.load(deps.storage, key.clone());
    match maybe_proxy_asset {
        Ok(asset) => Some(asset),
        Err(_) => {
            missing_assets.insert(key.to_string());
            None
        }
    }
}

fn get_value_ref_dependencies(value_reference: &ValueRef, entry: String) -> Vec<AssetEntry> {
    match value_reference {
        abstract_os::objects::proxy_asset::ValueRef::Pool { pair } => {
            // Check if the other asset in the pool resolves
            let other_pool_asset: AssetEntry = other_asset_name(entry.as_str(), &pair.contract)
                .unwrap()
                .into();
            vec![other_pool_asset]
        }
        abstract_os::objects::proxy_asset::ValueRef::LiquidityToken {} => {
            // check if both tokens of pool resolve
            let maybe_pair: UncheckedContractEntry = entry.try_into().unwrap();
            let other_pool_asset_names = get_pair_asset_names(maybe_pair.contract.as_str());
            let asset1: AssetEntry = other_pool_asset_names[0].into();
            let asset2: AssetEntry = other_pool_asset_names[1].into();
            vec![asset1, asset2]
        }
        abstract_os::objects::proxy_asset::ValueRef::Proxy {
            proxy_asset,
            multiplier: _,
        } => vec![proxy_asset.clone()],
        abstract_os::objects::proxy_asset::ValueRef::External { api_name: _ } => todo!(),
    }
}
