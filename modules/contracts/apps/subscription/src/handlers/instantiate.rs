use abstract_subscription_interface::state::subscription::INCOME_TWA;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{SubscriptionApp, SubscriptionResult},
    msg::SubscriptionInstantiateMsg,
    state::{SubscribersConfig, SubscriptionState, SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE},
};

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    _app: SubscriptionApp,
    msg: SubscriptionInstantiateMsg,
) -> SubscriptionResult {
    let subscription_config: SubscribersConfig = SubscribersConfig {
        payment_asset: msg.subscribers.payment_asset.check(deps.api, None)?,
        subscription_cost_per_block: msg.subscribers.subscription_cost_per_block,
        factory_address: deps.api.addr_validate(&msg.subscribers.factory_addr)?,
        subscription_per_block_emissions: msg
            .subscribers
            .subscription_per_block_emissions
            .check(deps.api)?,
    };

    let subscription_state: SubscriptionState = SubscriptionState { active_subs: 0 };

    // Optional contribution setup
    if let Some(msg) = msg.contributors {
        // TODO: install app on account
    }

    SUBSCRIPTION_CONFIG.save(deps.storage, &subscription_config)?;
    SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;
    INCOME_TWA.instantiate(
        deps.storage,
        &env,
        None,
        msg.subscribers.income_averaging_period.u64(),
    )?;

    Ok(Response::new())
}
