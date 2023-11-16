use crate::contract::{AppResult, PaymentApp};
use crate::error::AppError;
use crate::msg::TipCountResponse;
use crate::msg::TipperResponse;
use crate::msg::TippersResponse;
use crate::msg::{AppQueryMsg, ConfigResponse};
use crate::state::{Tipper, CONFIG, TIPPERS, TIP_COUNT};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult, Storage};
use cw_paginate::paginate_map;
use cw_storage_plus::Bound;

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
        AppQueryMsg::Tipper { address } => to_json_binary(&query_tipper(deps, address)?),
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

fn query_tip_count(deps: Deps) -> AppResult<TipCountResponse> {
    let count = TIP_COUNT.may_load(deps.storage)?.unwrap_or_default();
    Ok(TipCountResponse { count })
}

fn query_tipper(deps: Deps, address: String) -> AppResult<TipperResponse> {
    let address = deps.api.addr_validate(&address)?;
    let tipper: Option<Tipper> = TIPPERS.may_load(deps.storage, &address)?;
    if let Some(tipper) = tipper {
        Ok(TipperResponse {
            address,
            total_amount: tipper.amount,
            count: tipper.count,
        })
    } else {
        Err(AppError::TipperDoesNotExist {})
    }
}

fn query_list_tippers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> AppResult<TippersResponse> {
    let tippers = if let Some(start_after) = start_after {
        let addr = deps.api.addr_validate(&start_after)?;
        let start_after = Some(Bound::exclusive(&addr));
        paginate(deps.storage, start_after, limit)?
    } else {
        paginate(deps.storage, None, limit)?
    };

    Ok(TippersResponse { tippers })
}

fn paginate<'a>(
    storage: &dyn Storage,
    start_after: Option<Bound<'a, &'a Addr>>,
    limit: Option<u32>,
) -> Result<Vec<TipperResponse>, AppError> {
    paginate_map(
        &TIPPERS,
        storage,
        start_after,
        limit,
        |k, v| -> AppResult<_> {
            Ok(TipperResponse {
                address: k,
                total_amount: v.amount,
                count: v.count,
            })
        },
    )
}
