use abstract_std::objects::voting::DEFAULT_LIMIT;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdResult, Uint128};
use cw_asset::Asset;
use cw_storage_plus::Bound;

use crate::{
    contract::{SubscriptionApp, SubscriptionResult},
    msg::{
        StateResponse, SubscriberResponse, SubscribersResponse, SubscriptionFeeResponse,
        SubscriptionQueryMsg,
    },
    state::{
        EXPIRED_SUBSCRIBERS, INCOME_TWA, SUBSCRIBERS, SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE,
    },
};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    _app: &SubscriptionApp,
    msg: SubscriptionQueryMsg,
) -> SubscriptionResult<Binary> {
    match msg {
        // handle dapp-specific queries here
        SubscriptionQueryMsg::State {} => {
            let subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
            to_json_binary(&StateResponse {
                subscription: subscription_state,
            })
        }
        SubscriptionQueryMsg::Fee {} => {
            let config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
            let twa_data = INCOME_TWA.load(deps.storage)?;
            let minimal_cost =
                Uint128::from(twa_data.averaging_period) * config.subscription_cost_per_second;
            to_json_binary(&SubscriptionFeeResponse {
                fee: Asset {
                    info: config.payment_asset,
                    amount: minimal_cost,
                },
            })
        }
        SubscriptionQueryMsg::Config {} => {
            let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
            to_json_binary(&subscription_config)
        }
        SubscriptionQueryMsg::Subscriber { addr } => to_json_binary(&query_subscriber(deps, addr)?),
        SubscriptionQueryMsg::Subscribers {
            start_after,
            limit,
            expired_subs,
        } => to_json_binary(&query_subscribers(deps, start_after, limit, expired_subs)?),
    }
    .map_err(Into::into)
}

fn query_subscriber(deps: Deps, addr: String) -> SubscriptionResult<SubscriberResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    let subscription_state = if let Some(sub) = SUBSCRIBERS.may_load(deps.storage, &addr)? {
        SubscriberResponse {
            currently_subscribed: true,
            subscriber_details: Some(sub),
        }
    } else if let Some(sub) = EXPIRED_SUBSCRIBERS.may_load(deps.storage, &addr)? {
        SubscriberResponse {
            currently_subscribed: false,
            subscriber_details: Some(sub),
        }
    } else {
        SubscriberResponse {
            currently_subscribed: false,
            subscriber_details: None,
        }
    };
    Ok(subscription_state)
}

fn query_subscribers(
    deps: Deps,
    start_after: Option<cosmwasm_std::Addr>,
    limit: Option<u64>,
    expired_subs: Option<bool>,
) -> SubscriptionResult<SubscribersResponse> {
    let min = start_after.as_ref().map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let subscribed = !expired_subs.unwrap_or(false);
    let map = match subscribed {
        true => SUBSCRIBERS,
        false => EXPIRED_SUBSCRIBERS,
    };
    let subscribers = map
        .range(deps.storage, min, None, cosmwasm_std::Order::Ascending)
        .take(limit as usize)
        .map(|entry| {
            entry.map(|(addr, sub)| {
                (
                    addr,
                    SubscriberResponse {
                        currently_subscribed: subscribed,
                        subscriber_details: Some(sub),
                    },
                )
            })
        })
        .collect::<StdResult<_>>()?;
    Ok(SubscribersResponse { subscribers })
}
