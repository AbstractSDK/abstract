use abstract_subscription_interface::subscription::msg::SubscriptionInstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{SubscriptionApp, SubscriptionResult},
    state::{
        SubscriptionConfig, SubscriptionState, INCOME_TWA, SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE,
    },
};

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    _app: SubscriptionApp,
    msg: SubscriptionInstantiateMsg,
) -> SubscriptionResult {
    let subscription_config: SubscriptionConfig = SubscriptionConfig {
        payment_asset: msg.payment_asset.check(deps.api, None)?,
        subscription_cost_per_block: msg.subscription_cost_per_block,
        factory_address: deps.api.addr_validate(&msg.factory_addr)?,
        subscription_per_block_emissions: msg.subscription_per_block_emissions.check(deps.api)?,
        contributors_enabled: false,
    };

    let subscription_state: SubscriptionState = SubscriptionState { active_subs: 0 };
    SUBSCRIPTION_CONFIG.save(deps.storage, &subscription_config)?;
    SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;
    INCOME_TWA.instantiate(deps.storage, &env, None, msg.income_averaging_period.u64())?;

    Ok(Response::new())
}
