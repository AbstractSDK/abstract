use crate::contract::{AppResult, PaymentApp};
use crate::msg::TippersCountResponse;
use crate::msg::{AppQueryMsg, ConfigResponse};
use crate::msg::{TipAmountAtHeightResponse, TipperResponse};
use crate::msg::{TipCountResponse, TipperCountResponse};
use crate::state::{CONFIG, TIPPERS, TIPPER_COUNT, TIP_COUNT};
use abstract_core::objects::{AnsAsset, AssetEntry};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

const DEFAULT_LIMIT: u32 = 10;

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &PaymentApp,
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
        } => to_json_binary(&query_tipper(deps, address, start_after, limit)?),
        AppQueryMsg::TipAtHeight {
            address,
            asset,
            height,
        } => to_json_binary(&query_tip_at_height(deps, address, asset, height)?),
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

fn query_tip_at_height(
    deps: Deps,
    address: String,
    asset: AssetEntry,
    height: u64,
) -> AppResult<TipAmountAtHeightResponse> {
    let address = deps.api.addr_validate(&address)?;

    let amount = TIPPERS.may_load_at_height(deps.storage, (&address, &asset), height)?;
    Ok(TipAmountAtHeightResponse { amount })
}
