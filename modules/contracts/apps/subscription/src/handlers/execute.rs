use abstract_app::sdk::{
    cw_helpers::Clearable, AbstractResponse, AccountAction, Execution, TransferInterface,
};
use cosmwasm_std::{Addr, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw_asset::{Asset, AssetInfoUnchecked};

use crate::{
    contract::{SubscriptionApp, SubscriptionResult},
    msg::{SubscriptionExecuteMsg, UnsubscribedHookMsg},
    state::{
        EmissionType, Subscriber, SubscriptionConfig, SubscriptionState, EXPIRED_SUBSCRIBERS,
        INCOME_TWA, SUBSCRIBERS, SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE,
    },
    SubscriptionError,
};

pub(crate) const MAX_UNSUBS: usize = 15;

pub fn execute_handler(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: SubscriptionApp,
    msg: SubscriptionExecuteMsg,
) -> SubscriptionResult {
    match msg {
        SubscriptionExecuteMsg::Pay { subscriber_addr } => {
            let maybe_received_coin = info.funds.last();
            let subscriber_addr = subscriber_addr
                .map(|human| deps.api.addr_validate(&human))
                .transpose()?
                .unwrap_or(info.sender.clone());
            if let Some(coin) = maybe_received_coin.cloned() {
                try_pay(module, deps, env, Asset::from(coin), subscriber_addr)
            } else {
                Err(SubscriptionError::NotUsingCW20Hook {})
            }
        }
        SubscriptionExecuteMsg::Unsubscribe { unsubscribe_addrs } => {
            unsubscribe(deps, env, module, unsubscribe_addrs)
        }
        SubscriptionExecuteMsg::ClaimEmissions { addr } => {
            claim_subscriber_emissions(&module, &mut deps, &env, addr)
        }
        SubscriptionExecuteMsg::UpdateSubscriptionConfig {
            payment_asset,
            subscription_cost_per_second,
            subscription_per_second_emissions,
            unsubscribe_hook_addr,
        } => update_subscription_config(
            deps,
            env,
            info,
            module,
            payment_asset,
            subscription_cost_per_second,
            subscription_per_second_emissions,
            unsubscribe_hook_addr,
        ),
        SubscriptionExecuteMsg::RefreshTWA {} => {
            INCOME_TWA.try_update_value(&env, deps.storage)?;
            Ok(Response::new())
        }
    }
}

/// Called when either paying with a native token or through the receive_cw20 endpoint when paying
/// with a CW20.
pub fn try_pay(
    module: SubscriptionApp,
    deps: DepsMut,
    env: Env,
    asset: Asset,
    subscriber_addr: Addr,
) -> SubscriptionResult {
    // Load all needed states
    let config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
    let twa_data = INCOME_TWA.load(deps.storage)?;
    let base_state = module.load_state(deps.storage)?;
    // Construct deposit info
    let deposit_info = config.payment_asset;

    // Assert payment asset and claimed asset infos are the same
    if deposit_info != asset.info {
        return Err(SubscriptionError::WrongToken(deposit_info));
    }
    // Minimum of one period worth to (re)-subscribe.
    // prevents un- and re-subscribing all the time.
    let required_payment = Uint128::from(twa_data.averaging_period)
        .checked_mul_ceil(config.subscription_cost_per_second)?;
    let paid_for_seconds = asset
        .amount
        .checked_div_floor(config.subscription_cost_per_second)?
        .u128() as u64;
    if let Some(mut active_sub) = SUBSCRIBERS.may_load(deps.storage, &subscriber_addr)? {
        // Subscriber is active, update balance
        active_sub.extend(paid_for_seconds);
        SUBSCRIBERS.save(deps.storage, &subscriber_addr, &active_sub)?;
    } else {
        // Subscriber is (re)activating his subscription.
        if asset.amount < required_payment {
            return Err(SubscriptionError::InsufficientPayment(
                required_payment,
                deposit_info.to_string(),
            ));
        }
        let subscriber = Subscriber::new(&env.block, paid_for_seconds);
        let mut subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
        INCOME_TWA.accumulate(
            &env,
            deps.storage,
            Decimal::from_atomics(Uint128::from(subscription_state.active_subs), 0)?
                * config.subscription_cost_per_second,
        )?;
        // Remove from expired list in case it's re-sub
        EXPIRED_SUBSCRIBERS.remove(deps.storage, &subscriber_addr);

        SUBSCRIBERS.save(deps.storage, &subscriber_addr, &subscriber)?;
        subscription_state.active_subs += 1;
        SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;
    }

    Ok(module
        .response("pay")
        .add_attribute("received_funds", asset.to_string())
        .add_message(
            // Send the received asset to the proxy
            asset.transfer_msg(base_state.proxy_address)?,
        ))
}

pub fn unsubscribe(
    deps: DepsMut,
    env: Env,
    module: SubscriptionApp,
    unsubscribe_addrs: Vec<String>,
) -> SubscriptionResult {
    if unsubscribe_addrs.len() > MAX_UNSUBS {
        return Err(SubscriptionError::TooManyUnsubs {});
    }
    let unsubscribe_addrs: Vec<Addr> = unsubscribe_addrs
        .iter()
        .map(|human| deps.api.addr_validate(human))
        .collect::<StdResult<_>>()?;
    let mut subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
    let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
    let mut canceled_subs: Vec<String> = vec![];
    let mut claim_actions: Vec<AccountAction> = vec![];

    // update income
    INCOME_TWA.accumulate(
        &env,
        deps.storage,
        Decimal::from_atomics(Uint128::from(subscription_state.active_subs), 0)?
            * subscription_config.subscription_cost_per_second,
    )?;

    for addr in unsubscribe_addrs.into_iter() {
        let mut subscriber = SUBSCRIBERS.load(deps.storage, &addr)?;
        if subscriber.is_expired(&env.block) {
            let maybe_claim_msg = match claim_emissions_msg(
                &module,
                deps.as_ref(),
                &env,
                &mut subscriber,
                &addr,
                subscription_config
                    .subscription_per_second_emissions
                    .clone(),
                &subscription_state,
            ) {
                Ok(maybe_msg) => maybe_msg,
                // If just claimed or not enabled - no claims
                Err(SubscriptionError::EmissionsAlreadyClaimed {})
                | Err(SubscriptionError::SubscriberEmissionsNotEnabled {}) => None,
                Err(error) => {
                    return Err(error);
                }
            };

            subscription_state.active_subs -= 1;
            SUBSCRIBERS.remove(deps.storage, &addr);
            EXPIRED_SUBSCRIBERS.save(deps.storage, &addr, &subscriber)?;
            canceled_subs.push(addr.into_string());

            if let Some(msg) = maybe_claim_msg {
                claim_actions.push(msg)
            }
        }
    }

    // Error if no one unsubbed
    if canceled_subs.is_empty() {
        return Err(SubscriptionError::NoOneUnsubbed {});
    }

    // update subscription count
    SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;

    let mut response = module
        .response("unsubscribe")
        .add_messages(module.executor(deps.as_ref()).execute(claim_actions));

    if let Some(hook) = subscription_config.unsubscribe_hook_addr {
        let msg = UnsubscribedHookMsg {
            unsubscribed: canceled_subs,
        }
        .into_cosmos_msg(hook)?;
        response = response.add_message(msg);
    }

    Ok(response)
}

// Claim emissions
pub fn claim_emissions_msg(
    module: &SubscriptionApp,
    deps: Deps,
    env: &Env,
    subscriber: &mut Subscriber,
    subscriber_addr: &Addr,
    subscription_per_second_emissions: EmissionType<Addr>,
    subscription_state: &SubscriptionState,
) -> SubscriptionResult<Option<AccountAction>> {
    if subscriber.last_emission_claim_timestamp >= env.block.time {
        return Err(SubscriptionError::EmissionsAlreadyClaimed {});
    }

    let duration = env
        .block
        .time
        .minus_seconds(subscriber.last_emission_claim_timestamp.seconds());
    let seconds_passed = duration.seconds();

    let asset = match subscription_per_second_emissions {
        crate::state::EmissionType::None => {
            return Err(SubscriptionError::SubscriberEmissionsNotEnabled {});
        }
        crate::state::EmissionType::SecondShared(shared_emissions, token) => {
            // active_sub can't be 0 as we already loaded one from storage
            let amount = Uint128::from(seconds_passed).mul_floor(shared_emissions)
                / Uint128::from(subscription_state.active_subs);
            Asset::new(token, amount)
        }
        crate::state::EmissionType::SecondPerUser(per_user_emissions, token) => {
            let amount = Uint128::from(seconds_passed).mul_floor(per_user_emissions);
            Asset::new(token, amount)
        }
    };

    if !asset.amount.is_zero() {
        // Update only if there was claim
        subscriber.last_emission_claim_timestamp = env.block.time;

        let send_msg = module.bank(deps).transfer(vec![asset], subscriber_addr)?;
        Ok(Some(send_msg))
    } else {
        Ok(None)
    }
}

/// Checks if subscriber is allowed to claim his emissions
pub fn claim_subscriber_emissions(
    module: &SubscriptionApp,
    deps: &mut DepsMut,
    env: &Env,
    addr: String,
) -> SubscriptionResult {
    let subscriber_addr = deps.api.addr_validate(&addr)?;
    let subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
    let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
    let mut subscriber = SUBSCRIBERS.load(deps.storage, &subscriber_addr)?;

    let maybe_action = claim_emissions_msg(
        module,
        deps.as_ref(),
        env,
        &mut subscriber,
        &subscriber_addr,
        subscription_config.subscription_per_second_emissions,
        &subscription_state,
    )?;

    SUBSCRIBERS.save(deps.storage, &subscriber_addr, &subscriber)?;
    let mut response = module.response("claim_emissions");
    if let Some(action) = maybe_action {
        response = response.add_message(module.executor(deps.as_ref()).execute(vec![action])?);
    }
    Ok(response)
}

// Only Admin can execute it
#[allow(clippy::too_many_arguments)]
pub fn update_subscription_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    module: SubscriptionApp,
    payment_asset: Option<AssetInfoUnchecked>,
    subscription_cost_per_second: Option<Decimal>,
    subscription_per_second_emissions: Option<EmissionType<String>>,
    unsubscribe_hook_addr: Option<Clearable<String>>,
) -> SubscriptionResult {
    module
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;

    let mut config: SubscriptionConfig = SUBSCRIPTION_CONFIG.load(deps.storage)?;

    if let Some(subscription_cost_per_second) = subscription_cost_per_second {
        // validate address format
        config.subscription_cost_per_second = subscription_cost_per_second;
    }

    if let Some(payment_asset) = payment_asset {
        config.payment_asset = payment_asset.check(deps.api, None)?;
    }

    if let Some(subscription_per_second_emissions) = subscription_per_second_emissions {
        config.subscription_per_second_emissions =
            subscription_per_second_emissions.check(deps.api)?;
    }

    if let Some(clearable_hook_addr) = unsubscribe_hook_addr {
        config.unsubscribe_hook_addr = clearable_hook_addr.check(deps.api)?.into();
    }

    SUBSCRIPTION_CONFIG.save(deps.storage, &config)?;

    Ok(module.response("update_subscription_config"))
}
