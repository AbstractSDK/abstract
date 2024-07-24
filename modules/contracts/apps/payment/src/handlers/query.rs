use abstract_app::std::objects::{AnsAsset, AssetEntry};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

use crate::{
    contract::{AppResult, PaymentApp},
    msg::{
        AppQueryMsg, ConfigResponse, TipCountResponse, TipperCountResponse, TipperResponse,
        TippersCountResponse,
    },
    state::{CONFIG, TIPPERS, TIPPER_COUNT, TIP_COUNT},
};

const DEFAULT_LIMIT: u32 = 10;

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _module: &PaymentApp,
    msg: AppQueryMsg,
) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        AppQueryMsg::ListTippersCount { start_after, limit } => {
            to_json_binary(&query_list_tippers_count(deps, start_after, limit)?)
        }
        AppQueryMsg::TipCount {} => to_json_binary(&query_tip_count(deps)?),
        AppQueryMsg::Tipper {
            address,
            start_after,
            limit,
            at_height,
        } => to_json_binary(&query_tipper(deps, address, start_after, limit, at_height)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        desired_asset: config.desired_asset,
        denom_asset: config.denom_asset,
        exchanges: config.exchanges,
    })
}

fn query_tip_count(deps: Deps) -> AppResult<TipCountResponse> {
    let count = TIP_COUNT.may_load(deps.storage)?.unwrap_or_default();
    Ok(TipCountResponse { count })
}

fn query_tipper(
    deps: Deps,
    address: String,
    start_after: Option<AssetEntry>,
    limit: Option<u32>,
    at_height: Option<u64>,
) -> AppResult<TipperResponse> {
    let address = deps.api.addr_validate(&address)?;
    // Load tipper at height if provided
    if let Some(height) = at_height {
        return tipper_at_height(deps, address, start_after, limit, height);
    }
    let amounts = TIPPERS
        .prefix(&address)
        .range(
            deps.storage,
            start_after.as_ref().map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit.unwrap_or(DEFAULT_LIMIT) as usize)
        .map(|item| item.map(|(asset, amount)| AnsAsset::new(asset, amount)))
        .collect::<StdResult<_>>()?;

    let count = TIPPER_COUNT
        .may_load(deps.storage, &address)?
        .unwrap_or_default();
    Ok(TipperResponse {
        address,
        tip_count: count,
        total_amounts: amounts,
    })
}

fn query_list_tippers_count(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> AppResult<TippersCountResponse> {
    let start_after = start_after
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?;

    let tippers: Vec<_> = TIPPER_COUNT
        .range(
            deps.storage,
            start_after.as_ref().map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit.unwrap_or(DEFAULT_LIMIT) as usize)
        .map(|res| res.map(|(address, count)| TipperCountResponse { address, count }))
        .collect::<StdResult<_>>()?;

    Ok(TippersCountResponse { tippers })
}

fn tipper_at_height(
    deps: Deps,
    address: Addr,
    start_after: Option<AssetEntry>,
    limit: Option<u32>,
    height: u64,
) -> AppResult<TipperResponse> {
    // Load current keys for future queries
    let entries: Vec<AssetEntry> = TIPPERS
        .prefix(&address)
        .keys(
            deps.storage,
            start_after.as_ref().map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit.unwrap_or(DEFAULT_LIMIT) as usize)
        .collect::<StdResult<_>>()?;

    let total_amounts: Vec<AnsAsset> = entries
        .into_iter()
        .map(|entry| {
            let res = TIPPERS.may_load_at_height(deps.storage, (&address, &entry), height)?;
            Ok(AnsAsset {
                name: entry,
                amount: res.unwrap_or_default(),
            })
        })
        .collect::<AppResult<_>>()?;

    // Get count at height
    let count = TIPPER_COUNT
        .may_load_at_height(deps.storage, &address, height)?
        .unwrap_or_default();

    Ok(TipperResponse {
        address,
        tip_count: count,
        total_amounts,
    })
}
