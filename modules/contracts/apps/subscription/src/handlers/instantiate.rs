use cosmwasm_std::{ensure, DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{SubscriptionApp, SubscriptionResult},
    msg::SubscriptionInstantiateMsg,
    state::{
        SubscriptionConfig, SubscriptionState, INCOME_TWA, SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE,
    },
    SubscriptionError,
};

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    _module: SubscriptionApp,
    msg: SubscriptionInstantiateMsg,
) -> SubscriptionResult {
    let subscription_config: SubscriptionConfig = SubscriptionConfig {
        payment_asset: msg.payment_asset.check(deps.api, None)?,
        subscription_cost_per_second: msg.subscription_cost_per_second,
        subscription_per_second_emissions: msg.subscription_per_second_emissions.check(deps.api)?,
        unsubscribe_hook_addr: msg
            .unsubscribe_hook_addr
            .map(|human| deps.api.addr_validate(&human))
            .transpose()?,
    };

    let subscription_state: SubscriptionState = SubscriptionState { active_subs: 0 };
    SUBSCRIPTION_CONFIG.save(deps.storage, &subscription_config)?;
    SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;

    ensure!(
        !msg.income_averaging_period.is_zero(),
        SubscriptionError::ZeroAveragePeriod {}
    );
    INCOME_TWA.instantiate(deps.storage, &env, None, msg.income_averaging_period.u64())?;

    Ok(Response::new())
}
