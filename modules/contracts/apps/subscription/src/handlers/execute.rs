use crate::contract::{SubscriptionApp, SubscriptionResult};
use crate::msg::SubscriptionExecuteMsg;
use crate::state::{
    EmissionType, Subscriber, SubscriptionConfig, DORMANT_SUBSCRIBERS, INCOME_TWA, SUBSCRIBERS,
    SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE,
};
use crate::{SubscriptionError, DURATION_IN_WEEKS, WEEK_IN_SECONDS};
use abstract_sdk::core::manager::ExecuteMsg as ManagerMsg;
use abstract_sdk::{Execution, TransferInterface};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg,
    Uint128, WasmMsg,
};
use cw_asset::{Asset, AssetInfoUnchecked};

pub fn execute_handler(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: SubscriptionApp,
    msg: SubscriptionExecuteMsg,
) -> SubscriptionResult {
    match msg {
        SubscriptionExecuteMsg::Pay {
            subscriber_addr,
            unsubscribe_hook_addr,
        } => {
            let maybe_received_coin = info.funds.last();
            let subscriber_addr = subscriber_addr
                .map(|human| deps.api.addr_validate(&human))
                .transpose()?
                .unwrap_or(info.sender.clone());
            if let Some(coin) = maybe_received_coin.cloned() {
                try_pay(
                    app,
                    deps,
                    env,
                    info,
                    Asset::from(coin),
                    subscriber_addr,
                    unsubscribe_hook_addr,
                )
            } else {
                Err(SubscriptionError::NotUsingCW20Hook {})
            }
        }
        SubscriptionExecuteMsg::Unsubscribe { unsubscribe_addrs } => {
            unsubscribe(deps, env, app, unsubscribe_addrs)
        }
        SubscriptionExecuteMsg::ClaimEmissions { addr } => {
            claim_subscriber_emissions(&app, &mut deps, &env, addr)
        }
        SubscriptionExecuteMsg::UpdateSubscriptionConfig {
            payment_asset,
            subscription_cost_per_week: subscription_cost,
            subscription_per_week_emissions,
        } => update_subscription_config(
            deps,
            env,
            info,
            app,
            payment_asset,
            subscription_cost,
            subscription_per_week_emissions,
        ),
        SubscriptionExecuteMsg::RefreshTWA {} => {
            INCOME_TWA.try_update_value(&env, deps.storage)?;
            Ok(Response::new())
        }
    }
}

// ############
//  SUBSCRIPTION
// ############

/// Called when either paying with a native token or through the receive_cw20 endpoint when paying
/// with a CW20.
pub fn try_pay(
    app: SubscriptionApp,
    deps: DepsMut,
    env: Env,
    _msg_info: MessageInfo,
    asset: Asset,
    subscriber_addr: Addr,
    unsubscribe_hook_addr: Option<String>,
) -> SubscriptionResult {
    let unsubscribe_hook_addr = unsubscribe_hook_addr
        .map(|human| deps.api.addr_validate(&human))
        .transpose()?;
    // Load all needed states
    let config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
    let base_state = app.load_state(deps.storage)?;
    // Construct deposit info
    let deposit_info = config.payment_asset;

    // Assert payment asset and claimed asset infos are the same
    if deposit_info != asset.info {
        return Err(SubscriptionError::WrongToken(deposit_info));
    }
    // Init vector for logging
    let attrs = vec![
        ("action", String::from("Deposit to subscription module")),
        ("Received funds:", asset.to_string()),
    ];

    let maybe_subscriber = SUBSCRIBERS.may_load(deps.storage, &subscriber_addr)?;
    // Minimum of one period worth to (re)-subscribe.
    // prevents un- and re-subscribing all the time.
    let required_payment = Uint128::from(DURATION_IN_WEEKS) * config.subscription_cost_per_week;
    let paid_for_days = {
        // TODO: Decimals feels pretty annoying

        let paid_for_weeks = asset
            .amount
            .checked_div_floor(config.subscription_cost_per_week)?
            .u128() as u64;
        paid_for_weeks * 7
    };
    if let Some(mut active_sub) = maybe_subscriber {
        // Subscriber is active, update balance
        active_sub.expiration_timestamp = active_sub.expiration_timestamp.plus_days(paid_for_days);
        // Update hook addr if required
        // TODO: do we need a way to disable hook?
        if let Some(new_hook_addr) = unsubscribe_hook_addr {
            active_sub.unsubscribe_hook_addr = Some(new_hook_addr);
        }
        SUBSCRIBERS.save(deps.storage, &subscriber_addr, &active_sub)?;
    } else {
        // Subscriber is (re)activating his subscription.
        if asset.amount.u128() < required_payment.u128() {
            return Err(SubscriptionError::InsufficientPayment(
                required_payment.u128() as u64,
                deposit_info.to_string(),
            ));
        }
        let maybe_old_client = DORMANT_SUBSCRIBERS.may_load(deps.storage, &subscriber_addr)?;
        // if old client
        if let Some(mut old_client) = maybe_old_client {
            DORMANT_SUBSCRIBERS.remove(deps.storage, &subscriber_addr);
            old_client.expiration_timestamp = env.block.time.plus_days(paid_for_days);
            old_client.last_emission_claim_timestamp = env.block.time;
            // Update hook addr if required
            if let Some(new_hook_addr) = unsubscribe_hook_addr {
                old_client.unsubscribe_hook_addr = Some(new_hook_addr);
            }
            SUBSCRIBERS.save(deps.storage, &subscriber_addr, &old_client)?;
            return Ok(Response::new().add_attributes(attrs).add_message(
                // Send the received asset to the proxy
                asset.transfer_msg(base_state.proxy_address)?,
            ));
        } else {
            // New client
            let new_sub = Subscriber {
                expiration_timestamp: env.block.time.plus_days(paid_for_days),
                last_emission_claim_timestamp: env.block.time,
                unsubscribe_hook_addr,
            };
            SUBSCRIBERS.save(deps.storage, &subscriber_addr, &new_sub)?;
        }
        let mut subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
        INCOME_TWA.accumulate(
            &env,
            deps.storage,
            Decimal::from_atomics(Uint128::from(subscription_state.active_subs), 0)?
                * config.subscription_cost_per_week,
        )?;
        subscription_state.active_subs += 1;
        SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;
    }

    Ok(Response::new().add_attributes(attrs).add_message(
        // Send the received asset to the proxy
        asset.transfer_msg(base_state.proxy_address)?,
    ))
}

pub fn unsubscribe(
    mut deps: DepsMut,
    env: Env,
    app: SubscriptionApp,
    unsubscribe_addrs: Vec<String>,
) -> SubscriptionResult {
    let unsubscribe_addrs: Vec<Addr> = unsubscribe_addrs
        .iter()
        .map(|human| deps.api.addr_validate(human))
        .collect::<StdResult<_>>()?;
    let mut subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
    let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
    let mut suspend_msgs: Vec<SubMsg> = vec![];

    // update income
    INCOME_TWA.accumulate(
        &env,
        deps.storage,
        Decimal::from_atomics(Uint128::from(subscription_state.active_subs), 0)?
            * subscription_config.subscription_cost_per_week,
    )?;

    for addr in unsubscribe_addrs.iter() {
        let mut subscriber = SUBSCRIBERS.load(deps.storage, addr)?;
        // TODO:
        // contributors have free access
        // if CONTRIBUTORS.has(deps.storage, &subscriber.manager_addr) {
        //     continue;
        // }
        if let Some(mut msg) = expired_sub_msgs(&mut deps, &env, &mut subscriber, addr, &app)? {
            subscription_state.active_subs -= 1;
            SUBSCRIBERS.remove(deps.storage, addr);
            DORMANT_SUBSCRIBERS.save(deps.storage, addr, &subscriber)?;
            suspend_msgs.append(&mut msg);
        }
    }

    SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;
    Ok(Response::new().add_submessages(suspend_msgs))
}

/// Checks if subscriber is allowed to claim his emissions
pub fn claim_subscriber_emissions(
    app: &SubscriptionApp,
    deps: &mut DepsMut,
    env: &Env,
    addr: String,
) -> SubscriptionResult {
    let subscriber_addr = deps.api.addr_validate(&addr)?;
    let subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
    let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
    // TODO: this one is called during unsubscribe which means it won't have this addr as a key
    let mut subscriber = SUBSCRIBERS.load(deps.storage, &subscriber_addr)?;

    if subscriber.last_emission_claim_timestamp >= env.block.time {
        return Err(SubscriptionError::EmissionsAlreadyClaimed {});
    }

    let duration = env
        .block
        .time
        .minus_seconds(subscriber.last_emission_claim_timestamp.seconds());
    let weeks_passed = duration.seconds() / WEEK_IN_SECONDS;
    println!("weeks_passed: {weeks_passed}");

    let asset = match subscription_config.subscription_per_week_emissions {
        crate::state::EmissionType::None => {
            return Err(SubscriptionError::SubscriberEmissionsNotEnabled {});
        }
        crate::state::EmissionType::WeekShared(shared_emissions, token) => {
            // active_sub can't be 0 as we already loaded from storage
            let amount = (shared_emissions * Uint128::from(weeks_passed))
                / (Uint128::from(subscription_state.active_subs));
            Asset::new(token, amount)
        }
        crate::state::EmissionType::WeekPerUser(per_user_emissions, token) => {
            let amount = per_user_emissions * Uint128::from(weeks_passed);
            Asset::new(token, amount)
        } // crate::state::EmissionType::IncomeBased(token) => {
          //     if !subscription_config.contributors_enabled {
          //         return Err(SubscriptionError::ContributionNotEnabled {});
          //     }
          //     let contributors_addr = app.modules(deps.as_ref()).module_address(CONTRIBUTORS_ID)?;
          //     let contributor_config =
          //         contr_state::CONTRIBUTION_CONFIG.query(&deps.querier, contributors_addr.clone())?;
          //     let contributor_state =
          //         contr_state::CONTRIBUTION_STATE.query(&deps.querier, contributors_addr)?;

          //     let amount = (contributor_state.emissions * contributor_config.emission_user_share)
          //         / Uint128::from(subscription_state.active_subs);
          //     Asset::new(token, amount * Uint128::from(1u64))
          // }
    };

    println!("asset: {asset}");
    if !asset.amount.is_zero() {
        // Update only if there was claim
        subscriber.last_emission_claim_timestamp = env.block.time;
        SUBSCRIBERS.save(deps.storage, &subscriber_addr, &subscriber)?;

        let send_msg = app
            .bank(deps.as_ref())
            .transfer(vec![asset], &subscriber_addr)?;
        Ok(Response::new().add_message(app.executor(deps.as_ref()).execute(vec![send_msg])?))
    } else {
        Ok(Response::new())
    }
}

// ############
//  CONFIGS
// ############

// Only Admin can execute it
#[allow(clippy::too_many_arguments)]
pub fn update_subscription_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: SubscriptionApp,
    payment_asset: Option<AssetInfoUnchecked>,
    subscription_cost_per_week: Option<Decimal>,
    subscription_per_week_emissions: Option<EmissionType<String>>,
) -> SubscriptionResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config: SubscriptionConfig = SUBSCRIPTION_CONFIG.load(deps.storage)?;

    if let Some(subscription_cost_per_week) = subscription_cost_per_week {
        // validate address format
        config.subscription_cost_per_week = subscription_cost_per_week;
    }

    if let Some(payment_asset) = payment_asset {
        config.payment_asset = payment_asset.check(deps.api, None)?;
    }

    if let Some(subscription_per_week_emissions) = subscription_per_week_emissions {
        config.subscription_per_week_emissions = subscription_per_week_emissions.check(deps.api)?;
    }

    SUBSCRIPTION_CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_subscriber_config"))
}

/// Check if expired
/// if so, generate emission msg and suspend manager
fn expired_sub_msgs(
    deps: &mut DepsMut,
    env: &Env,
    subscriber: &mut Subscriber,
    unsubscriber_addr: &Addr,
    app: &SubscriptionApp,
) -> Result<Option<Vec<SubMsg>>, SubscriptionError> {
    if subscriber.expiration_timestamp <= env.block.time {
        // TODO: claim emissions before the un-sub
        let mut resp = claim_subscriber_emissions(app, deps, env, unsubscriber_addr.to_string())?;
        // TODO: add hooks instead
        // resp = resp.add_message(suspend_os(subscriber.manager_addr.clone(), true)?);
        return Ok(Some(resp.messages));
    }
    Ok(None)
}

pub fn suspend_os(manager_address: Addr, new_suspend_status: bool) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: manager_address.to_string(),
        msg: to_binary(&ManagerMsg::UpdateStatus {
            is_suspended: Some(new_suspend_status),
        })?,
        funds: vec![],
    }))
}
