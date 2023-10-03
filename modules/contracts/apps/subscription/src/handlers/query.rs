use abstract_subscription_interface::DURATION_IN_WEEKS;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, Uint128};
use cw_asset::Asset;

use crate::{
    contract::{SubscriptionApp, SubscriptionResult},
    msg::{StateResponse, SubscriberStateResponse, SubscriptionFeeResponse, SubscriptionQueryMsg},
    state::{DORMANT_SUBSCRIBERS, SUBSCRIBERS, SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE},
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
            to_binary(&StateResponse {
                subscription: subscription_state,
            })
        }
        SubscriptionQueryMsg::Fee {} => {
            let config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
            let minimal_cost = Uint128::from(DURATION_IN_WEEKS) * config.subscription_cost_per_week;
            to_binary(&SubscriptionFeeResponse {
                fee: Asset {
                    info: config.payment_asset,
                    amount: minimal_cost,
                },
            })
        }
        SubscriptionQueryMsg::Config {} => {
            let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
            to_binary(&subscription_config)
        }
        SubscriptionQueryMsg::SubscriberState { os_id } => {
            let maybe_sub = SUBSCRIBERS.may_load(deps.storage, &os_id)?;
            let maybe_dormant_sub = DORMANT_SUBSCRIBERS.may_load(deps.storage, &os_id)?;
            let subscription_state = if let Some(sub) = maybe_sub {
                to_binary(&SubscriberStateResponse {
                    currently_subscribed: true,
                    subscriber_details: sub,
                })?
            } else if let Some(sub) = maybe_dormant_sub {
                to_binary(&SubscriberStateResponse {
                    currently_subscribed: true,
                    subscriber_details: sub,
                })?
            } else {
                return Err(StdError::generic_err("os has os_id 0 or does not exist").into());
            };
            Ok(subscription_state)
        }
    }
    .map_err(Into::into)
}
