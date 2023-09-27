use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    contract::{SubscriptionApp, SubscriptionResult},
    msg::SubscriptionInstantiateMsg,
    state::{
        ContributionState, ContributorsConfig, SubscribersConfig, SubscriptionState,
        CONTRIBUTION_CONFIG, CONTRIBUTION_STATE, INCOME_TWA, SUBSCRIPTION_CONFIG,
        SUBSCRIPTION_STATE,
    },
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
        let contributor_config: ContributorsConfig = ContributorsConfig {
            emissions_amp_factor: msg.emissions_amp_factor,
            emission_user_share: msg.emission_user_share,
            emissions_offset: msg.emissions_offset,
            protocol_income_share: msg.protocol_income_share,
            max_emissions_multiple: msg.max_emissions_multiple,
            token_info: msg.token_info.check(deps.api, None)?,
        }
        .verify()?;

        let contributor_state: ContributionState = ContributionState {
            income_target: Decimal::zero(),
            expense: Decimal::zero(),
            total_weight: Uint128::zero(),
            emissions: Decimal::zero(),
        };
        CONTRIBUTION_CONFIG.save(deps.storage, &contributor_config)?;
        CONTRIBUTION_STATE.save(deps.storage, &contributor_state)?;
        INCOME_TWA.instantiate(deps.storage, &env, None, msg.income_averaging_period.u64())?;
    }

    SUBSCRIPTION_CONFIG.save(deps.storage, &subscription_config)?;
    SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;

    Ok(Response::new())
}
