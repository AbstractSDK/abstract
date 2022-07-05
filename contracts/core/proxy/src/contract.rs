use abstract_os::objects::core::OS_ID;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response,
    StdResult, Uint128,
};

use crate::error::ProxyError;
use abstract_os::objects::proxy_assets::{get_asset_identifier, ProxyAsset};
use abstract_os::proxy::state::{State, ADMIN, STATE, VAULT_ASSETS};
use abstract_os::proxy::{
    ConfigResponse, ExecuteMsg, HoldingAmountResponse, HoldingValueResponse, InstantiateMsg,
    MigrateMsg, QueryMsg, TotalValueResponse, VaultAssetConfigResponse,
};
use abstract_os::PROXY;
use cw2::{get_contract_version, set_contract_version};
use cw_asset::AssetInfo;
use semver::Version;
type ProxyResult = Result<Response, ProxyError>;
/*
    The proxy is the bank account of the protocol. It owns the liquidity and acts as a proxy contract.
    Whitelisted dApps construct messages for this contract. The dApps are controlled by Governance.
*/
// TODO: test max limit on-chain
const LIST_SIZE_LIMIT: usize = 15;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
            // TODO
            // let to_add = to_add.iter().map(From::from).collect();
            // update_assets(deps, info, to_add, to_remove)
            Ok(Response::default())
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
    to_add: Vec<ProxyAsset>,
    to_remove: Vec<AssetInfo>,
) -> ProxyResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    // Check the vault size to be within the size limit to prevent running out of gas when doing lookups
    let current_vault_size = VAULT_ASSETS
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    let delta: i128 = to_add.len() as i128 - to_remove.len() as i128;
    if current_vault_size as i128 + delta > LIST_SIZE_LIMIT as i128 {
        return Err(ProxyError::AssetsLimitReached {});
    }

    for new_asset in to_add.into_iter() {
        let id = get_asset_identifier(&new_asset.asset.info);
        // update function for new or existing keys
        let insert =
            |_vault_asset: Option<ProxyAsset>| -> StdResult<ProxyAsset> { Ok(new_asset.clone()) };
        VAULT_ASSETS.update(deps.storage, &id, insert)?;
    }

    for asset_id in to_remove {
        VAULT_ASSETS.remove(deps.storage, get_asset_identifier(&asset_id).as_str());
    }

    Ok(Response::new().add_attribute("action", "update_cw20_token_list"))
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
        QueryMsg::TotalValue {} => to_binary(&TotalValueResponse {
            value: compute_total_value(deps, env)?,
        }),
        QueryMsg::HoldingAmount { identifier } => {
            let vault_asset: ProxyAsset = VAULT_ASSETS.load(deps.storage, identifier.as_str())?;
            to_binary(&HoldingAmountResponse {
                value: vault_asset
                    .asset
                    .info
                    .query_balance(&deps.querier, env.contract.address)?,
            })
        }
        QueryMsg::HoldingValue { identifier } => to_binary(&HoldingValueResponse {
            value: compute_holding_value(deps, &env, identifier)?,
        }),
        QueryMsg::VaultAssetConfig { identifier } => to_binary(&VaultAssetConfigResponse {
            value: VAULT_ASSETS.load(deps.storage, identifier.as_str())?,
        }),
    }
}

/// Returns the whitelisted modules
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = STATE.load(deps.storage)?;
    let modules: Vec<Addr> = state.modules;
    let resp = ConfigResponse {
        modules: modules
            .iter()
            .map(|module| -> String { module.to_string() })
            .collect(),
    };
    Ok(resp)
}

/// Returns the value of a specified asset.
pub fn compute_holding_value(deps: Deps, env: &Env, holding: String) -> StdResult<Uint128> {
    let mut vault_asset: ProxyAsset = VAULT_ASSETS.load(deps.storage, holding.as_str())?;
    let value = vault_asset.value(deps, env, None)?;
    Ok(value)
}

/// Computes the total value locked in this contract
pub fn compute_total_value(deps: Deps, env: Env) -> StdResult<Uint128> {
    // Get all assets from storage
    let mut all_assets = VAULT_ASSETS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<(String, ProxyAsset)>>>()?;

    let mut total_value = Uint128::zero();
    // Calculate their value iteratively
    for vault_asset_entry in all_assets.iter_mut() {
        total_value += vault_asset_entry.1.value(deps, &env, None)?;
    }

    Ok(total_value)
}
