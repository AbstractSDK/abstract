use crate::contract::{SubscriptionApp, SubscriptionResult, BLOCKS_PER_MONTH};
use crate::msg::SubscriptionExecuteMsg;
use crate::state::{
    Subscriber, SubscriptionConfig, DORMANT_SUBSCRIBERS, INCOME_TWA, SUBSCRIBERS,
    SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE,
};
use abstract_core::objects::AccountId;
use abstract_sdk::{AccountVerification, Execution, ModuleInterface, TransferInterface};
use abstract_subscription_interface::utils::suspend_os;
use abstract_subscription_interface::{SubscriptionError, CONTRIBUTORS_ID};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    SubMsg, Uint128, WasmMsg,
};
use cw_asset::{Asset, AssetInfoUnchecked};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    app: SubscriptionApp,
    msg: SubscriptionExecuteMsg,
) -> SubscriptionResult {
    match msg {
        SubscriptionExecuteMsg::Pay { os_id } => {
            let maybe_received_coin = info.funds.last();
            if let Some(coin) = maybe_received_coin.cloned() {
                try_pay(app, deps, env, info, Asset::from(coin), os_id)
            } else {
                Err(SubscriptionError::NotUsingCW20Hook {})
            }
        }
        SubscriptionExecuteMsg::Unsubscribe { os_ids } => unsubscribe(deps, env, app, os_ids),
        SubscriptionExecuteMsg::ClaimEmissions { os_id } => {
            claim_subscriber_emissions(&app, deps.as_ref(), &env, &os_id)
        }
        SubscriptionExecuteMsg::UpdateSubscriptionConfig {
            payment_asset,
            factory_address,
            subscription_cost_per_block: subscription_cost,
            contributors_enabled,
        } => update_subscription_config(
            deps,
            env,
            info,
            app,
            payment_asset,
            factory_address,
            subscription_cost,
            contributors_enabled,
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
    msg_info: MessageInfo,
    asset: Asset,
    os_id: AccountId,
) -> SubscriptionResult {
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

    let maybe_subscriber = SUBSCRIBERS.may_load(deps.storage, &os_id)?;
    // Minimum of one month's worth to (re)-subscribe.
    // prevents un- and re-subscribing all the time.
    let required_payment = Uint128::from(BLOCKS_PER_MONTH) * config.subscription_cost_per_block;
    let paid_for_blocks = (asset.amount * config.subscription_cost_per_block).u128() as u64;
    if let Some(mut active_sub) = maybe_subscriber {
        // Subscriber is active, update balance
        active_sub.expiration_block += paid_for_blocks;
        SUBSCRIBERS.save(deps.storage, &os_id, &active_sub)?;
    } else {
        // Subscriber is (re)activating his subscription.
        if asset.amount.u128() < required_payment.u128() {
            return Err(SubscriptionError::InsufficientPayment(
                required_payment.u128() as u64,
                deposit_info.to_string(),
            ));
        }
        let maybe_old_client = DORMANT_SUBSCRIBERS.may_load(deps.storage, &os_id)?;
        // if old client
        if let Some(mut old_client) = maybe_old_client {
            DORMANT_SUBSCRIBERS.remove(deps.storage, &os_id);
            old_client.expiration_block = env.block.height + paid_for_blocks;
            old_client.last_emission_claim_block = env.block.height;
            SUBSCRIBERS.save(deps.storage, &os_id, &old_client)?;
            return Ok(Response::new()
                .add_attributes(attrs)
                // Unsuspend subscriber
                .add_message(suspend_os(old_client.manager_addr, false)?)
                .add_message(
                    // Send the received asset to the proxy
                    asset.transfer_msg(base_state.proxy_address)?,
                ));
        } else {
            // New client
            // only factory can add subscribers
            if msg_info.sender != config.factory_address {
                return Err(SubscriptionError::CallerNotFactory {});
            }
            let manager_addr = app
                .account_registry(deps.as_ref())
                .account_base(&os_id)?
                .manager;
            let new_sub = Subscriber {
                expiration_block: env.block.height + paid_for_blocks,
                last_emission_claim_block: env.block.height,
                manager_addr,
            };
            SUBSCRIBERS.save(deps.storage, &os_id, &new_sub)?;
        }
        let mut subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
        subscription_state.active_subs += 1;
        SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;
        INCOME_TWA.accumulate(
            &env,
            deps.storage,
            Decimal::from_atomics(Uint128::from(subscription_state.active_subs), 0)?
                * config.subscription_cost_per_block,
        )?;
    }

    Ok(Response::new().add_attributes(attrs).add_message(
        // Send the received asset to the proxy
        asset.transfer_msg(base_state.proxy_address)?,
    ))
}

pub fn unsubscribe(
    deps: DepsMut,
    env: Env,
    app: SubscriptionApp,
    os_ids: Vec<AccountId>,
) -> SubscriptionResult {
    let mut subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
    let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
    let mut suspend_msgs: Vec<SubMsg> = vec![];
    for os_id in os_ids {
        let mut subscriber = SUBSCRIBERS.load(deps.storage, &os_id)?;
        // TODO:
        // contributors have free access
        // if CONTRIBUTORS.has(deps.storage, &subscriber.manager_addr) {
        //     continue;
        // }
        if let Some(mut msg) = expired_sub_msgs(deps.as_ref(), &env, &mut subscriber, &os_id, &app)?
        {
            subscription_state.active_subs -= 1;
            SUBSCRIBERS.remove(deps.storage, &os_id);
            DORMANT_SUBSCRIBERS.save(deps.storage, &os_id, &subscriber)?;
            suspend_msgs.append(&mut msg);
        }
    }

    SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;
    // update income
    INCOME_TWA.accumulate(
        &env,
        deps.storage,
        Decimal::from_atomics(Uint128::from(subscription_state.active_subs), 0)?
            * subscription_config.subscription_cost_per_block,
    )?;
    Ok(Response::new().add_submessages(suspend_msgs))
}

/// Checks if subscriber is allowed to claim his emissions
pub fn claim_subscriber_emissions(
    app: &SubscriptionApp,
    deps: Deps,
    env: &Env,
    os_id: &AccountId,
) -> SubscriptionResult {
    let subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
    let subscription_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
    let mut subscriber = SUBSCRIBERS.load(deps.storage, os_id)?;

    let subscriber_proxy_address = app.account_registry(deps).account_base(os_id)?.proxy;

    if subscriber.last_emission_claim_block >= env.block.height {
        return Err(SubscriptionError::EmissionsAlreadyClaimed {});
    }

    let duration = env.block.height - subscriber.last_emission_claim_block;
    subscriber.last_emission_claim_block = env.block.height;

    let asset = match subscription_config.subscription_per_block_emissions {
        crate::state::EmissionType::None => {
            return Err(SubscriptionError::SubscriberEmissionsNotEnabled)
        }
        crate::state::EmissionType::BlockShared(shared_emissions, token) => {
            // active_sub can't be 0 as we already loaded from storage
            let amount = (shared_emissions * Uint128::from(duration))
                / (Uint128::from(subscription_state.active_subs));
            Asset::new(token, amount)
        }
        crate::state::EmissionType::BlockPerUser(per_user_emissions, token) => {
            let amount = per_user_emissions * Uint128::from(duration);
            Asset::new(token, amount)
        }
        crate::state::EmissionType::IncomeBased(token) => {
            todo!()
            // TODO:
            // let contributor_config = load_contribution_config(deps.storage)?;
            // let contributor_state = CONTRIBUTION_STATE.load(deps.storage)?;

            // let amount = (contributor_state.emissions * contributor_config.emission_user_share)
            //     / Uint128::from(subscription_state.active_subs);
            // Asset::new(token, amount * Uint128::from(1u64))
        }
    };

    if !asset.amount.is_zero() {
        let send_msg = app
            .bank(deps)
            .transfer(vec![asset], &subscriber_proxy_address)?;
        Ok(Response::new().add_message(app.executor(deps).execute(vec![send_msg])?))
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
    factory_address: Option<String>,
    subscription_cost_per_block: Option<Decimal>,
    contributors_enabled: Option<bool>,
) -> SubscriptionResult {
    // Let contributors contract self-enable
    if let (Some(true), None, None, None) = (
        &contributors_enabled,
        &payment_asset,
        &factory_address,
        &subscription_cost_per_block,
    ) {
        let contributos_addr = app.modules(deps.as_ref()).module_address(CONTRIBUTORS_ID)?;
        if info.sender == contributos_addr {
            SUBSCRIPTION_CONFIG.update(deps.storage, |mut config| {
                config.contributors_enabled = true;
                StdResult::Ok(config)
            })?;
            return Ok(Response::new().add_attribute("action", "update_subscriber_config"));
        }
    }

    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config: SubscriptionConfig = SUBSCRIPTION_CONFIG.load(deps.storage)?;

    if let Some(factory_address) = factory_address {
        // validate address format
        config.factory_address = deps.api.addr_validate(&factory_address)?;
    }

    if let Some(subscription_cost_per_block) = subscription_cost_per_block {
        // validate address format
        config.subscription_cost_per_block = subscription_cost_per_block;
    }

    if let Some(payment_asset) = payment_asset {
        config.payment_asset = payment_asset.check(deps.api, None)?;
    }

    if let Some(contributors_enabled) = contributors_enabled {
        config.contributors_enabled = contributors_enabled;
    }

    SUBSCRIPTION_CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_subscriber_config"))
}

/// Check if expired
/// if so, generate emission msg and suspend manager
fn expired_sub_msgs(
    deps: Deps,
    env: &Env,
    subscriber: &mut Subscriber,
    os_id: &AccountId,
    app: &SubscriptionApp,
) -> Result<Option<Vec<SubMsg>>, SubscriptionError> {
    if subscriber.expiration_block <= env.block.height {
        let mut resp = claim_subscriber_emissions(app, deps, env, os_id)?;
        resp = resp.add_message(suspend_os(subscriber.manager_addr.clone(), true)?);
        return Ok(Some(resp.messages));
    }
    Ok(None)
}
