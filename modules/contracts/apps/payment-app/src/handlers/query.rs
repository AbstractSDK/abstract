use crate::contract::{AppResult, PaymentApp};
use crate::msg::TipCountResponse;
use crate::msg::TipperResponse;
use crate::msg::TippersResponse;
use crate::msg::{AppQueryMsg, ConfigResponse};
use crate::state::{CONFIG, TIPPERS, TIP_COUNT};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw_paginate::paginate_map;
use cw_storage_plus::Bound;

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &PaymentApp,
    msg: AppQueryMsg,
) -> AppResult<Binary> {
    match msg {
        AppQueryMsg::Config {} => to_binary(&query_config(deps)?),
        AppQueryMsg::ListTippers { start_after, limit } => {
            to_binary(&list_tippers(deps, start_after, limit)?)
        }
        AppQueryMsg::TipCount {} => to_binary(&tip_count(deps)?),
        AppQueryMsg::Tipper { address } => to_binary(&tipper(deps, address)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        desired_asset: config.desired_asset,
        exchanges: config.exchanges,
    })
}

fn tip_count(deps: Deps) -> AppResult<TipCountResponse> {
    let count = TIP_COUNT.load(deps.storage).unwrap_or(0);
    Ok(TipCountResponse { count })
}

fn tipper(deps: Deps, address: String) -> AppResult<TipperResponse> {
    let address = deps.api.addr_validate(&address)?;
    let tipper = TIPPERS.load(deps.storage, address.clone())?;
    Ok(TipperResponse {
        address,
        total_amount: tipper.amount,
        count: tipper.count,
    })
}

fn list_tippers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> AppResult<TippersResponse> {
    let start_after = start_after
        .map(|s| deps.api.addr_validate(&s))
        .transpose()?
        .map(Bound::exclusive);
    let tippers = paginate_map(
        &TIPPERS,
        deps.storage,
        start_after,
        limit,
        |k, v| -> AppResult<_> {
            Ok(TipperResponse {
                address: k,
                total_amount: v.amount,
                count: v.count,
            })
        },
    )?;

    Ok(TippersResponse { tippers })
}
