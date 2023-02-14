use crate::commands::*;
use crate::error::ProxyError;
use crate::queries::*;
use abstract_os::objects::module_version::migrate_module_data;

use abstract_sdk::{
    feature_objects::AnsHost,
    os::{
        objects::{core::OS_ID, module_version::set_module_data, AssetEntry},
        proxy::{
            state::{State, ADMIN, ANS_HOST, STATE, VAULT_ASSETS},
            AssetConfigResponse, ExecuteMsg, HoldingValueResponse, InstantiateMsg, MigrateMsg,
            QueryMsg,
        },
        PROXY,
    },
};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

pub type ProxyResult<T = Response> = Result<T, ProxyError>;
/*
    The proxy is the bank account of the protocol. It owns the liquidity and acts as a proxy contract.
    Whitelisted dApps construct messages for this contract. The dApps are controlled by Governance.
*/
// TODO: test max limit on-chain
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ProxyResult {
    // Use CW2 to set the contract version, this is needed for migrations
    set_contract_version(deps.storage, PROXY, CONTRACT_VERSION)?;
    set_module_data(deps.storage, PROXY, CONTRACT_VERSION, &[], None::<String>)?;
    OS_ID.save(deps.storage, &msg.os_id)?;
    STATE.save(deps.storage, &State { modules: vec![] })?;
    ANS_HOST.save(
        deps.storage,
        &AnsHost {
            address: deps.api.addr_validate(&msg.ans_host_address)?,
        },
    )?;
    let admin_addr = Some(info.sender);
    ADMIN.set(deps, admin_addr)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ProxyResult {
    match msg {
        ExecuteMsg::ModuleAction { msgs } => execute_module_action(deps, info, msgs),
        ExecuteMsg::IbcAction { msgs } => execute_ibc_action(deps, info, msgs),
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, &admin),
        ExecuteMsg::AddModule { module } => add_module(deps, info, module),
        ExecuteMsg::RemoveModule { module } => remove_module(deps, info, module),
        ExecuteMsg::UpdateAssets { to_add, to_remove } => {
            update_assets(deps, info, to_add, to_remove)
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ProxyResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();

    if storage_version < version {
        set_contract_version(deps.storage, PROXY, CONTRACT_VERSION)?;
        migrate_module_data(deps.storage, PROXY, CONTRACT_VERSION, None::<String>)?;
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> ProxyResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::TotalValue {} => to_binary(&query_total_value(deps, env)?),
        QueryMsg::HoldingAmount { identifier } => {
            to_binary(&query_holding_amount(deps, env, identifier)?)
        }
        QueryMsg::TokenValue { identifier, amount } => {
            to_binary(&query_token_value(deps, env, identifier, amount)?)
        }
        QueryMsg::HoldingValue { identifier } => to_binary(&HoldingValueResponse {
            value: compute_holding_value(deps, &env, identifier)?,
        }),
        QueryMsg::AssetConfig { identifier } => to_binary(&AssetConfigResponse {
            proxy_asset: VAULT_ASSETS.load(deps.storage, &AssetEntry::from(identifier))?,
        }),
        QueryMsg::Assets { start_after, limit } => {
            to_binary(&query_proxy_assets(deps, start_after, limit)?)
        }
        QueryMsg::CheckValidity {} => to_binary(&query_proxy_asset_validity(deps)?),
        QueryMsg::BaseAsset {} => to_binary(&query_base_asset(deps)?),
    }
    .map_err(Into::into)
}
