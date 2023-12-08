use crate::contract::{AppResult, PaymentApp};
use crate::msg::TipCountResponse;
use crate::msg::TipperResponse;
use crate::msg::TippersResponse;
use crate::msg::{AppQueryMsg, ConfigResponse};
use crate::state::{CONFIG, TIPPERS, TIP_COUNT};
use abstract_core::objects::{AnsAsset, AssetEntry};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult};
use cw_paginate::paginate_map_prefix;
use cw_storage_plus::{Bound, PrefixBound};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &PaymentApp,
    msg: AppQueryMsg,
) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        AppQueryMsg::ListTippers { start_after, limit } => {
            to_json_binary(&query_list_tippers(deps, start_after, limit)?)
        }
        AppQueryMsg::TipCount {} => to_json_binary(&query_tip_count(deps)?),
        AppQueryMsg::Tipper {
            address,
            start_after,
            limit,
        } => to_json_binary(&query_tipper(deps, address, start_after, limit)?),
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
) -> AppResult<TipperResponse> {
    let address = deps.api.addr_validate(&address)?;
    let amounts = paginate_map_prefix(
        &TIPPERS,
        deps.storage,
        &address,
        start_after.as_ref().map(Bound::exclusive),
        limit,
        |asset, amount| AppResult::Ok(AnsAsset::new(asset, amount)),
    )?;
    Ok(TipperResponse {
        address,
        total_amounts: amounts,
    })
}

// TODO: make map indexedmap and return only addrs
fn query_list_tippers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> AppResult<TippersResponse> {
    let start_after = start_after
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?;

    let tippers: Vec<_> = TIPPERS
        .prefix_range(
            deps.storage,
            start_after.as_ref().map(PrefixBound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit.unwrap_or(cw_paginate::DEFAULT_LIMIT) as usize)
        .map(|res| {
            res.map(|((addr, asset), amount)| TipperResponse {
                address: addr,
                total_amounts: vec![AnsAsset::new(asset, amount)],
            })
        })
        .collect::<StdResult<_>>()?;

    Ok(TippersResponse { tippers })
}
