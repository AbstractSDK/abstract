use crate::contract::{SubscriptionApp, SubscriptionResult, BLOCKS_PER_MONTH};
use crate::error::SubscriptionError;
use crate::msg::SubscriptionExecuteMsg;
use crate::state::{
    Compensation, ContributionState, ContributorsConfig, Subscriber, SubscribersConfig,
    CACHED_CONTRIBUTION_STATE, CONTRIBUTION_CONFIG, CONTRIBUTION_STATE, CONTRIBUTORS,
    DORMANT_SUBSCRIBERS, INCOME_TWA, SUBSCRIBERS, SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE,
};
use abstract_core::objects::AccountId;
use abstract_sdk::core::manager::state::ACCOUNT_ID;
use abstract_sdk::core::manager::ExecuteMsg as ManagerMsg;
use abstract_sdk::core::version_control::AccountBase;
use abstract_sdk::{AccountVerification, Execution, TransferInterface};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Storage, SubMsg, Uint128, Uint64, WasmMsg,
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
        SubscriptionExecuteMsg::ClaimCompensation { os_id } => {
            try_claim_compensation(app, deps, env, os_id)
        }
        SubscriptionExecuteMsg::ClaimEmissions { os_id } => {
            claim_subscriber_emissions(&app, deps.as_ref(), &env, &os_id)
        }
        SubscriptionExecuteMsg::UpdateContributor {
            os_id: contributor_os_id,
            base_per_block,
            weight,
            expiration_block,
        } => update_contributor_compensation(
            deps,
            env,
            info,
            app,
            contributor_os_id,
            base_per_block,
            weight.map(|w| w.u64() as u32),
            expiration_block.map(|w| w.u64()),
        ),
        SubscriptionExecuteMsg::RemoveContributor { os_id } => {
            remove_contributor(deps, info, app, os_id)
        }
        SubscriptionExecuteMsg::UpdateSubscriptionConfig {
            payment_asset,
            factory_address,
            subscription_cost_per_block: subscription_cost,
        } => update_subscription_config(
            deps,
            env,
            info,
            app,
            payment_asset,
            factory_address,
            subscription_cost,
        ),
        SubscriptionExecuteMsg::UpdateContributionConfig {
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            project_token_info,
            emissions_amp_factor,
            emissions_offset,
        } => update_contribution_config(
            deps,
            env,
            info,
            app,
            protocol_income_share,
            emission_user_share,
            max_emissions_multiple,
            project_token_info,
            emissions_amp_factor,
            emissions_offset,
        ),
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
        // contributors have free access
        if CONTRIBUTORS.has(deps.storage, &subscriber.manager_addr) {
            continue;
        }
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

fn suspend_os(manager_address: Addr, new_suspend_status: bool) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: manager_address.to_string(),
        msg: to_binary(&ManagerMsg::UpdateStatus {
            is_suspended: Some(new_suspend_status),
        })?,
        funds: vec![],
    }))
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
            let contributor_config = load_contribution_config(deps.storage)?;
            let contributor_state = CONTRIBUTION_STATE.load(deps.storage)?;

            let amount = (contributor_state.emissions * contributor_config.emission_user_share)
                / Uint128::from(subscription_state.active_subs);
            Asset::new(token, amount * Uint128::from(1u64))
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

// #################### //
//      CONTRIBUTION    //
// #################### //

/// Function that adds/updates the contributor config of a given address
#[allow(clippy::too_many_arguments)]
pub fn update_contributor_compensation(
    deps: DepsMut,
    env: Env,
    msg_info: MessageInfo,
    app: SubscriptionApp,
    contributor_os_id: AccountId,
    base_per_block: Option<Decimal>,
    weight: Option<u32>,
    expiration_block: Option<u64>,
) -> SubscriptionResult {
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let _config = load_contribution_config(deps.storage)?;
    // Load all needed states
    let mut state = CONTRIBUTION_STATE.load(deps.storage)?;
    let contributor_addr = app
        .account_registry(deps.as_ref())
        .account_base(&contributor_os_id)?
        .manager;

    let maybe_compensation = CONTRIBUTORS.may_load(deps.storage, &contributor_addr)?;

    let new_compensation = match maybe_compensation {
        Some(current_compensation) => {
            // Can only update if already claimed last period.
            let twa_income = INCOME_TWA.load(deps.storage)?;
            if current_compensation.last_claim_block.u64() < twa_income.last_averaging_block_height
            {
                return try_claim_compensation(app, deps, env, contributor_os_id);
            };
            let compensation =
                current_compensation
                    .clone()
                    .overwrite(base_per_block, weight, expiration_block);
            if current_compensation.base_per_block > compensation.base_per_block {
                let (base_diff, weight_diff) = current_compensation.clone() - compensation.clone();
                state.total_weight = Uint128::from(
                    (state.total_weight.u128() as i128 - weight_diff as i128) as u128,
                );
                state.income_target -= base_diff;
            } else {
                let (base_diff, weight_diff) = compensation.clone() - current_compensation.clone();
                state.total_weight = Uint128::from(
                    (state.total_weight.u128() as i128 + weight_diff as i128) as u128,
                );
                state.income_target += base_diff;
            };
            Compensation {
                base_per_block: compensation.base_per_block,
                weight: compensation.weight,
                expiration_block: compensation.expiration_block,
                ..current_compensation
            }
        }
        None => {
            let compensation =
                Compensation::default().overwrite(base_per_block, weight, expiration_block);

            let os_id = ACCOUNT_ID
                .query(&deps.querier, contributor_addr.clone())
                .map_err(|_| SubscriptionError::ContributorNotManager)?;
            let subscriber = SUBSCRIBERS.load(deps.storage, &os_id)?;
            if subscriber.manager_addr != contributor_addr {
                return Err(SubscriptionError::ContributorNotManager);
            }
            // New contributor doesn't pay for subscription but should be able to use os
            let mut subscription_state = SUBSCRIPTION_STATE.load(deps.storage)?;
            subscription_state.active_subs -= 1;
            SUBSCRIPTION_STATE.save(deps.storage, &subscription_state)?;
            // Move to dormant. Prevents them from claiming user emissions
            SUBSCRIBERS.remove(deps.storage, &os_id);
            DORMANT_SUBSCRIBERS.save(deps.storage, &os_id, &subscriber)?;
            state.total_weight += Uint128::from(compensation.weight);
            state.income_target += compensation.base_per_block;
            Compensation {
                base_per_block: compensation.base_per_block,
                weight: compensation.weight,
                expiration_block: compensation.expiration_block,
                last_claim_block: env.block.height.into(),
            }
        }
    };

    CONTRIBUTORS.save(deps.storage, &contributor_addr, &new_compensation)?;
    CONTRIBUTION_STATE.save(deps.storage, &state)?;

    // Init vector for logging
    let attrs = vec![
        ("action", String::from("update_compensation")),
        ("for", contributor_addr.to_string()),
    ];

    Ok(Response::new().add_attributes(attrs))
}

/// Removes the specified contributor
pub fn remove_contributor(
    deps: DepsMut,
    msg_info: MessageInfo,
    app: SubscriptionApp,
    os_id: AccountId,
) -> SubscriptionResult {
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let manager_address = app
        .account_registry(deps.as_ref())
        .account_base(&os_id)?
        .manager;
    remove_contributor_from_storage(deps.storage, manager_address.clone())?;
    // He must re-activate to join active set and earn emissions
    let msg = suspend_os(manager_address.clone(), true)?;
    // Init vector for logging
    let attrs = vec![
        ("action", String::from("remove_contributor")),
        ("address:", manager_address.to_string()),
    ];

    Ok(Response::new().add_message(msg).add_attributes(attrs))
}

// Check income
// Compute total contribution emissions
// Compute share of those emissions
// Compute share of income
/// Calculate the compensation for contribution
pub fn try_claim_compensation(
    app: SubscriptionApp,
    deps: DepsMut,
    env: Env,
    os_id: AccountId,
) -> SubscriptionResult {
    let config = load_contribution_config(deps.storage)?;
    let mut state = CONTRIBUTION_STATE.load(deps.storage)?;
    // Update contribution state if income changes
    let maybe_new_income = INCOME_TWA.try_update_value(&env, deps.storage)?;
    if let Some(income) = maybe_new_income {
        // Cache current state. This state will be used to pay out contributors of last period.
        CACHED_CONTRIBUTION_STATE.save(deps.storage, &state)?;
        // Overwrite current state with new income.
        update_contribution_state(deps.storage, env, &mut state, &config, income)?;
    }

    let cached_state = match CACHED_CONTRIBUTION_STATE.may_load(deps.storage)? {
        Some(state) => state,
        None => return Err(SubscriptionError::AveragingPeriodNotPassed),
    };

    if cached_state.income_target.is_zero() {
        return Err(SubscriptionError::TargetIsZero);
    };

    let contributor_emissions = match SUBSCRIPTION_CONFIG
        .load(deps.storage)?
        .subscription_per_block_emissions
    {
        crate::state::EmissionType::IncomeBased(_) => {
            cached_state.emissions * (Decimal::one() - config.emission_user_share)
        }
        _ => cached_state.emissions,
    };

    let AccountBase {
        manager: contributor_address,
        proxy: contributor_proxy_address,
    } = app.account_registry(deps.as_ref()).account_base(&os_id)?;

    let mut compensation = CONTRIBUTORS.load(deps.storage, &contributor_address)?;
    let twa_data = INCOME_TWA.load(deps.storage)?;

    if compensation.last_claim_block.u64() >= twa_data.last_averaging_block_height {
        // Already claimed previous period
        return Err(SubscriptionError::CompensationAlreadyClaimed);
    };

    let payable_blocks =
        if twa_data.last_averaging_block_height > compensation.expiration_block.u64() {
            // End of last period is after the expiration
            // Pay period between last claim and expiration
            remove_contributor_from_storage(deps.storage, contributor_address)?;
            compensation.expiration_block - compensation.last_claim_block
        } else {
            // pay full period
            let period =
                Uint64::from(twa_data.last_averaging_block_height) - compensation.last_claim_block;
            // update compensation details
            compensation.last_claim_block = twa_data.last_averaging_block_height.into();
            CONTRIBUTORS.save(deps.storage, &contributor_address, &compensation)?;
            period
        };

    // Payout depends on how much income was earned over that period.
    let payout_ratio = cached_state.expense / cached_state.income_target;
    // Pay period between last claim and end of cached state.
    let base_amount: Uint128 =
        (compensation.base_per_block * payout_ratio) * Uint128::from(payable_blocks);
    // calculate token emissions
    let token_amount = if !cached_state.total_weight.is_zero() {
        contributor_emissions
            * Decimal::from_ratio(compensation.weight as u128, cached_state.total_weight)
    } else {
        Decimal::zero()
    };

    let sub_config = SUBSCRIPTION_CONFIG.load(deps.storage)?;
    let mut assets = vec![];
    // Construct msgs
    if !base_amount.is_zero() {
        let base_asset: Asset = Asset::new(sub_config.payment_asset, base_amount);
        assets.push(base_asset);
    }

    if !token_amount.is_zero() {
        let token_asset: Asset = Asset::new(config.token_info, token_amount * Uint128::from(1u32));
        assets.push(token_asset)
    }
    if assets.is_empty() {
        Err(SubscriptionError::NoAssetsToSend)
    } else {
        let bank = app.bank(deps.as_ref());
        let transfer_action = bank.transfer(assets, &contributor_proxy_address)?;
        Ok(Response::new()
            .add_message(app.executor(deps.as_ref()).execute(vec![transfer_action])?)
            .add_attribute("action", "claim_contribution"))
    }
}

/// Update the contribution state
/// Call when income,target or config changes
fn update_contribution_state(
    store: &mut dyn Storage,
    _env: Env,
    contributor_state: &mut ContributionState,
    contributor_config: &ContributorsConfig,
    income: Decimal,
) -> StdResult<()> {
    let floor_emissions: Decimal =
        (Decimal::from_atomics(contributor_config.emissions_amp_factor, 0)
            .map_err(|e| StdError::GenericErr { msg: e.to_string() })?
            / contributor_state.income_target)
            + Decimal::from_atomics(contributor_config.emissions_offset, 0)
                .map_err(|e| StdError::GenericErr { msg: e.to_string() })?;
    let max_emissions = floor_emissions * contributor_config.max_emissions_multiple;
    if income < contributor_state.income_target {
        contributor_state.emissions = max_emissions
            - (max_emissions - floor_emissions) * (income / contributor_state.income_target);
        contributor_state.expense = income;
    } else {
        contributor_state.expense = contributor_state.income_target;
        contributor_state.emissions = floor_emissions;
    }
    CONTRIBUTION_STATE.save(store, contributor_state)
}

fn remove_contributor_from_storage(
    store: &mut dyn Storage,
    contributor_addr: Addr,
) -> StdResult<()> {
    // Load all needed states
    let mut state = CONTRIBUTION_STATE.load(store)?;

    let maybe_compensation = CONTRIBUTORS.may_load(store, &contributor_addr)?;

    match maybe_compensation {
        Some(current_compensation) => {
            state.total_weight -= Uint128::from(current_compensation.weight);
            state.income_target -= current_compensation.base_per_block;
            CONTRIBUTORS.remove(store, &contributor_addr);
            CONTRIBUTION_STATE.save(store, &state)?;
        }
        None => {
            return Err(StdError::GenericErr {
                msg: "contributor is not registered".to_string(),
            })
        }
    };
    Ok(())
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
) -> SubscriptionResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config: SubscribersConfig = SUBSCRIPTION_CONFIG.load(deps.storage)?;

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

    SUBSCRIPTION_CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_subscriber_config"))
}

// Only Admin can execute it
#[allow(clippy::too_many_arguments)]
pub fn update_contribution_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: SubscriptionApp,
    protocol_income_share: Option<Decimal>,
    emission_user_share: Option<Decimal>,
    max_emissions_multiple: Option<Decimal>,
    token_info: Option<AssetInfoUnchecked>,
    emissions_amp_factor: Option<Uint128>,
    emissions_offset: Option<Uint128>,
) -> SubscriptionResult {
    app.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config = CONTRIBUTION_CONFIG.load(deps.storage)?;

    if let Some(protocol_income_share) = protocol_income_share {
        config.protocol_income_share = protocol_income_share;
    }

    if let Some(emission_user_share) = emission_user_share {
        config.emission_user_share = emission_user_share;
    }

    if let Some(max_emissions_multiple) = max_emissions_multiple {
        config.max_emissions_multiple = max_emissions_multiple;
    }

    if let Some(emissions_amp_factor) = emissions_amp_factor {
        config.emissions_amp_factor = emissions_amp_factor;
    }

    if let Some(token_info) = token_info {
        // validate address format
        config.token_info = token_info.check(deps.api, None)?;
    }

    if let Some(emissions_offset) = emissions_offset {
        // validate address format
        config.emissions_offset = emissions_offset;
    }

    CONTRIBUTION_CONFIG.save(deps.storage, &config.verify()?)?;

    Ok(Response::new().add_attribute("action", "update contribution config"))
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

fn load_contribution_config(store: &dyn Storage) -> Result<ContributorsConfig, SubscriptionError> {
    // Check if user is using contribution feature
    let maybe_config = CONTRIBUTION_CONFIG.may_load(store)?;
    match maybe_config {
        Some(config) => Ok(config),
        None => Err(SubscriptionError::ContributionNotEnabled),
    }
}
