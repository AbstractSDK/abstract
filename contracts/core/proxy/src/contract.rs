use abstract_sdk::os::objects::core::OS_ID;
use abstract_sdk::os::objects::AssetEntry;

use abstract_sdk::feature_objects::AnsHost;
use abstract_sdk::Resolve;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response, StdError,
    StdResult, Uint128,
};

use crate::commands::*;
use crate::error::ProxyError;
use crate::queries::*;
use abstract_sdk::os::objects::proxy_asset::ProxyAsset;
use abstract_sdk::os::proxy::state::{State, ADMIN, ANS_HOST, STATE, VAULT_ASSETS};
use abstract_sdk::os::proxy::{
    AssetConfigResponse, BaseAssetResponse, ExecuteMsg, HoldingAmountResponse,
    HoldingValueResponse, InstantiateMsg, MigrateMsg, QueryMsg, TokenValueResponse,
    TotalValueResponse,
};
use abstract_sdk::os::PROXY;
use cw2::{get_contract_version, set_contract_version};
use semver::Version;
pub type ProxyResult = Result<Response, ProxyError>;
/*
    The proxy is the bank account of the protocol. It owns the liquidity and acts as a proxy contract.
    Whitelisted dApps construct messages for this contract. The dApps are controlled by Governance.
*/
// TODO: test max limit on-chain
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ProxyResult {
    match msg {
        ExecuteMsg::ModuleAction { msgs } => execute_action(deps, info, msgs),
        ExecuteMsg::IbcAction { msgs } => execute_ibc_action(deps, info, msgs),
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::TotalValue {} => to_binary(&TotalValueResponse {
            value: compute_total_value(deps, env)?,
        }),
        QueryMsg::HoldingAmount { identifier } => {
            let vault_asset: AssetEntry = identifier.into();
            let ans_host = ANS_HOST.load(deps.storage)?;
            let asset_info = vault_asset.resolve(&deps.querier, &ans_host)?;
            to_binary(&HoldingAmountResponse {
                amount: asset_info.query_balance(&deps.querier, env.contract.address)?,
            })
        }
        QueryMsg::TokenValue { identifier, amount } => to_binary(&TokenValueResponse {
            // Default the value calculation to one so that the caller doesn't need to provide a default
            value: compute_token_value(deps, &env, identifier, amount.or(Some(Uint128::one())))?,
        }),
        QueryMsg::HoldingValue { identifier } => to_binary(&HoldingValueResponse {
            value: compute_holding_value(deps, &env, identifier)?,
        }),
        QueryMsg::AssetConfig { identifier } => to_binary(&AssetConfigResponse {
            proxy_asset: VAULT_ASSETS.load(deps.storage, identifier.into())?,
        }),
        QueryMsg::Assets {
            page_token,
            page_size,
        } => to_binary(&query_proxy_assets(deps, page_token, page_size)?),
        QueryMsg::CheckValidity {} => to_binary(&query_proxy_asset_validity(deps)?),
        QueryMsg::BaseAsset {} => {
            let res: Result<Vec<(AssetEntry, ProxyAsset)>, _> = VAULT_ASSETS
                .range(deps.storage, None, None, Order::Ascending)
                .collect();
            let maybe_base_asset: Vec<(AssetEntry, ProxyAsset)> = res?
                .into_iter()
                .filter(|(_, p)| p.value_reference.is_none())
                .collect();
            if maybe_base_asset.len() != 1 {
                Err(StdError::generic_err("No base asset configured."))
            } else {
                to_binary(&BaseAssetResponse {
                    base_asset: maybe_base_asset[0].1.to_owned(),
                })
            }
        }
    }
}
