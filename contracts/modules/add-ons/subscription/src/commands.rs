use std::convert::TryInto;

use abstract_os::common_module::constants::ADMIN;
use abstract_os::core::manager::msg::ExecuteMsg as ManagerMsg;
use abstract_os::core::proxy::msg::send_to_proxy;
use cosmwasm_std::{
    from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, DepsMut, Empty, Env,
    MessageInfo, Response, StdError, StdResult, Storage, Uint128, Uint64, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_asset::{Asset, AssetInfo};
use cw_storage_plus::U32Key;

use abstract_os::native::version_control::state::OS_ADDRESSES;
use abstract_os::util::deposit_manager::Deposit;

use crate::contract::{SubscriptionAddOn, SubscriptionResult};
use crate::error::SubscriptionError;
use abstract_os::modules::add_ons::subscription::msg::DepositHookMsg;
use abstract_os::modules::add_ons::subscription::state::{
    Compensation, ContributionState, ContributorContext, IncomeAccumulator, Subscriber,
    SubscriberContext, SubscriptionConfig, SubscriptionState, CLIENTS, CONTRIBUTORS, CON_CONFIG,
    CON_STATE, DORMANT_CLIENTS, MONTH, SUB_CONFIG, SUB_STATE,
};

pub fn receive_cw20(
    add_on: SubscriptionAddOn,
    deps: DepsMut,
    _env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> SubscriptionResult {
    match from_binary(&cw20_msg.msg)? {
        DepositHookMsg::Pay { os_id } => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Cw20(msg_info.sender.clone()),
                amount: cw20_msg.amount,
            };
            try_pay(add_on, deps, msg_info, asset, os_id)
        }
    }
}

// ############
//  SUBSCRIPTION
// ############

/// Called when either paying with a native token or through the receive_cw20 endpoint when paying
/// with a CW20.
pub fn try_pay(
    add_on: SubscriptionAddOn,
    deps: DepsMut,
    msg_info: MessageInfo,
    asset: Asset,
    os_id: u32,
) -> SubscriptionResult {
    // Load all needed states
    let config = SUB_CONFIG.load(deps.storage)?;
    let base_state = add_on.base_state.load(deps.storage)?;

    // Construct deposit info
    let deposit_info = config.payment_asset;

    // Assert payment asset and claimed asset infos are the same
    if deposit_info != asset.info {
        return Err(SubscriptionError::WrongToken(deposit_info));
    }
    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Deposit to subscription module")),
        ("Received funds:", asset.to_string()),
    ];

    let maybe_subscriber = CLIENTS.may_load(deps.storage, &os_id.to_be_bytes())?;
    if let Some(mut active_sub) = maybe_subscriber {
        // Subscriber is active, update balance
        active_sub
            .balance
            .increase((asset.amount.u128() as u64).into());
        CLIENTS.save(deps.storage, &os_id.to_be_bytes(), &active_sub)?;
    } else {
        let maybe_old_client = DORMANT_CLIENTS.may_load(deps.storage, U32Key::new(os_id))?;
        if let Some(mut old_client) = maybe_old_client {
            // Subscriber is re-activating his subscription.
            if old_client.balance.get().u64() as u128 + asset.amount.u128()
                < config.subscription_cost.u64() as u128
            {
                return Err(SubscriptionError::InsufficientPayment(
                    config.subscription_cost.u64() - old_client.balance.get().u64(),
                ));
            } else {
                DORMANT_CLIENTS.remove(deps.storage, U32Key::new(os_id));
                old_client
                    .balance
                    .increase((asset.amount.u128() as u64).into());
                old_client.claimed_emissions = true;
                CLIENTS.save(deps.storage, &os_id.to_be_bytes(), &old_client)?;
                return Ok(Response::new()
                    .add_attributes(attrs)
                    // Unsuspend subscriber
                    .add_message(suspend_os(old_client.manager_addr, false)?)
                    .add_message(
                        // Send the received asset to the proxy
                        asset.transfer_msg(base_state.proxy_address)?,
                    ));
            }
        } else if (asset.amount.u128() as u64) < config.subscription_cost.u64() {
            return Err(SubscriptionError::InsufficientPayment(
                config.subscription_cost.u64(),
            ));
        } else {
            // New OS
            if msg_info.sender != config.factory_address {
                return Err(SubscriptionError::CallerNotFactory {});
            }
            let manager_addr = OS_ADDRESSES
                .query(
                    &deps.querier,
                    config.version_control_address,
                    U32Key::new(os_id),
                )?
                .unwrap()
                .manager;
            let new_sub = Subscriber {
                balance: Deposit::new().increase((asset.amount.u128() as u64).into()),
                claimed_emissions: true,
                manager_addr,
            };
            CLIENTS.save(deps.storage, &os_id.to_be_bytes(), &new_sub)?;
        }
    }

    Ok(Response::new().add_attributes(attrs).add_message(
        // Send the received asset to the proxy
        asset.transfer_msg(base_state.proxy_address)?,
    ))
}

/// First step in allowing contributors to claim their compensation.
/// 1. If past payment date, collect all income and log all unsubscribes
/// 2. Move all unsubscribed users from active to dormant. This prevents this operation from becoming prohibitory expensive.
/// 3. Pay out emissions to the remaining active subscribers
pub fn collect_subscriptions(
    mut deps: DepsMut,
    env: Env,
    page_limit: Option<u32>,
) -> SubscriptionResult {
    let sub_state = SUB_STATE.load(deps.storage)?;
    let mut con_state = CON_STATE.load(deps.storage)?;
    let con_config = CON_CONFIG.load(deps.storage)?;
    let sub_config = SUB_CONFIG.load(deps.storage)?;

    if con_state.next_pay_day.u64() <= env.block.time.seconds() {
        let response = Response::new();
        // First collect income
        if !sub_state.collected {
            let context = SubscriberContext {
                subscription_cost: sub_config.subscription_cost,
            };
            let (maybe_income, suspend_msgs) = collect_income(deps.branch(), page_limit, &context)?;
            if let Some(income) = maybe_income {
                update_contribution_state(
                    deps.as_ref(),
                    Uint64::new(
                        (Uint128::new(income as u128)
                            * (Decimal::one() - con_config.protocol_income_share))
                            .u128() as u64,
                    ),
                    &mut con_state,
                )?;
                CON_STATE.save(deps.storage, &con_state)?;
                return Ok(response
                    .add_messages(suspend_msgs)
                    .add_attribute("Status:", "Income collected"));
            } else {
                return Ok(response
                    .add_messages(suspend_msgs)
                    .add_attributes(vec![("Action:", String::from("Collecting income"))]));
            }
        };
        // Subscriptions are now collected and contribution state is updated
        // Now move all unsubscribed users

        // All unsubscribes are now handled. Set next collection date which enables contribution claiming.
        CON_STATE.update::<_, StdError>(deps.storage, |con| {
            Ok(ContributionState {
                next_pay_day: con.next_pay_day + Uint64::new(MONTH),
                ..con
            })
        })?;
        SUB_STATE.update::<_, StdError>(deps.storage, |sub| {
            Ok(SubscriptionState {
                collected: false,
                ..sub
            })
        })?;
        Ok(Response::new())
    } else {
        Err(SubscriptionError::WaitForNextPayday(
            con_state.next_pay_day.u64(),
        ))
    }
}

/// Uses accumulator page mapping to process all active subscribers
/// Returns Some() when operation is complete
fn collect_income(
    mut deps: DepsMut,
    page_limit: Option<u32>,
    context: &SubscriberContext,
) -> StdResult<(Option<u64>, Vec<CosmosMsg>)> {
    let (acc, suspend_msgs) =
        CLIENTS.page_with_accumulator(deps.branch(), page_limit, context, process_client)?;

    if let Some(result) = acc {
        let new_state = SUB_STATE.update::<_, StdError>(deps.storage, |_| {
            Ok(SubscriptionState {
                income: Uint64::from(result.income),
                active_subs: result.active_subs,
                collected: true,
            })
        })?;
        Ok((Some(new_state.income.u64()), suspend_msgs))
    } else {
        Ok((None, suspend_msgs))
    }
}

/// Check client payment
/// Allowed to make use of unsafe functions as this is the pagination function
fn process_client(
    key: &[u8],
    store: &mut dyn Storage,
    mut subscriber: Subscriber,
    acc: &mut IncomeAccumulator,
    context: &SubscriberContext,
) -> StdResult<Option<CosmosMsg>> {
    let subscription_cost = context.subscription_cost;

    match subscriber.balance.decrease(subscription_cost).ok() {
        Some(_) => {
            acc.income += subscription_cost.u64() as u32;
            acc.active_subs += 1;
            subscriber.claimed_emissions = false;
            CLIENTS.unsafe_save(store, key, &subscriber)?;
            Ok(None)
        }
        None => {
            // Contributors have free OS usage
            if CONTRIBUTORS.has(store, key) {
                return Ok(None);
            }
            let os_id = u32::from_be_bytes(key.to_owned().try_into().unwrap());
            acc.debtors.push(os_id);
            let removed_sub = CLIENTS.unsafe_remove(store, key)?;
            DORMANT_CLIENTS.save(store, os_id.into(), &removed_sub)?;
            Ok(Some(suspend_os(subscriber.manager_addr, true)?))
        }
    }
}
fn suspend_os(manager_address: Addr, new_suspend_status: bool) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: manager_address.to_string(),
        msg: to_binary(&ManagerMsg::SuspendOs {
            new_status: new_suspend_status,
        })?,
        funds: vec![],
    }))
}

fn update_contribution_state(
    deps: Deps,
    income: Uint64,
    con_state: &mut ContributionState,
) -> StdResult<()> {
    let contribution_config = CON_CONFIG.load(deps.storage)?;
    let floor_emissions = Decimal::from_ratio(
        contribution_config.emissions_amp_factor,
        Uint128::new(con_state.target.u64() as u128) + contribution_config.emissions_offset,
    );
    let max_emissions =
        floor_emissions * (contribution_config.max_emissions_multiple * Uint128::new(1));
    if income < con_state.target {
        con_state.emissions = max_emissions
            - (max_emissions - floor_emissions * Uint128::new(1))
                * Uint128::new((income / con_state.target).u64() as u128);
        con_state.expense = income;
    } else {
        con_state.expense = con_state.target;
        con_state.emissions = floor_emissions * Uint128::new(1);
    }
    Ok(())
}

/// Checks if subscriber is allowed to claim his emissions
pub fn claim_subscriber_emissions(
    add_on: SubscriptionAddOn,
    deps: DepsMut,
    env: Env,
    os_id: u32,
) -> SubscriptionResult {
    let sub_state = SUB_STATE.load(deps.storage)?;
    let con_state = CON_STATE.load(deps.storage)?;
    let con_config = CON_CONFIG.load(deps.storage)?;
    let sub_config = SUB_CONFIG.load(deps.storage)?;
    let mut subscriber = CLIENTS.load(deps.storage, &os_id.to_be_bytes())?;

    // Can only claim if current time is before pay day
    if env.block.time.seconds() > con_state.next_pay_day.u64()
        || CLIENTS.status.load(deps.storage)?.is_locked
    {
        return Err(SubscriptionError::CollectIncomeFirst);
    }

    let subscriber_proxy_address = OS_ADDRESSES
        .query(
            &deps.querier,
            sub_config.version_control_address,
            U32Key::new(os_id),
        )?
        .unwrap()
        .proxy;

    if subscriber.claimed_emissions {
        return Err(SubscriptionError::EmissionsAlreadyClaimed {});
    }
    subscriber.claimed_emissions = true;

    let token_amount = (con_state.emissions * con_config.emission_user_share).u128()
        / sub_state.active_subs as u128;
    let proxy_msg = send_to_proxy(
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: con_config.project_token.into_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: subscriber_proxy_address.into_string(),
                amount: token_amount.into(),
            })?,
            funds: vec![],
        })],
        &add_on.state(deps.storage)?.proxy_address,
    )?;

    if token_amount != 0 {
        Ok(Response::new().add_message(proxy_msg))
    } else {
        Ok(Response::new())
    }
}

// ############
//  CONTRIBUTION
// ############

/// Function that adds/updates the contributor config of a given address
pub fn update_contributor(
    deps: DepsMut,
    msg_info: MessageInfo,
    contributor_addr: String,
    compensation: Compensation,
) -> SubscriptionResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    // Load all needed states
    let mut state = CON_STATE.load(deps.storage)?;
    let maybe_compensation = CONTRIBUTORS.may_load(deps.storage, contributor_addr.as_bytes())?;

    let new_compensation = match maybe_compensation {
        Some(current_compensation) => {
            let (base_diff, weight_diff) = current_compensation.clone() - compensation.clone();
            // let base_diff: i32 = current_compensation.base as i32 - compensation.base as i32;
            // let weight_diff: i32 = current_compensation.weight as i32 - compensation.weight as i32;
            state.total_weight =
                Uint128::from((state.total_weight.u128() as i128 + weight_diff as i128) as u128);
            state.target = Uint64::from((state.target.u64() as i64 + base_diff as i64) as u64);
            Compensation {
                base: compensation.base,
                weight: compensation.weight,
                expiration: compensation.expiration,
                ..current_compensation
            }
        }
        None => {
            state.total_weight += Uint128::from(compensation.weight);
            state.target += Uint64::from(compensation.base);
            // Can only get paid on pay day after next pay day
            Compensation {
                base: compensation.base,
                weight: compensation.weight,
                expiration: compensation.expiration,
                next_pay_day: state.next_pay_day + Uint64::from(MONTH),
            }
        }
    };

    CONTRIBUTORS.save(deps.storage, contributor_addr.as_bytes(), &new_compensation)?;
    CON_STATE.save(deps.storage, &state)?;

    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Update Compensation")),
        ("For:", contributor_addr.to_string()),
    ];

    Ok(Response::new().add_attributes(attrs))
}

/// Removes the specified contributor
pub fn remove_contributor(
    deps: DepsMut,
    msg_info: MessageInfo,
    contributor_addr: String,
) -> SubscriptionResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    remove_contributor_from_storage(deps.storage, contributor_addr.as_bytes())?;
    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Remove Contributor")),
        ("Address:", contributor_addr),
    ];

    Ok(Response::new().add_attributes(attrs))
}

pub fn try_claim_contribution(
    add_on: SubscriptionAddOn,
    mut deps: DepsMut,
    env: Env,
    contributor_addr: Option<String>,
    page_limit: Option<u32>,
) -> SubscriptionResult {
    let state = CON_STATE.load(deps.storage)?;
    let config = CON_CONFIG.load(deps.storage)?;
    // let subscription_state = SUB_STATE.load(deps.storage)?;
    let base_state = add_on.state(deps.storage)?;
    let response = Response::new();

    if state.target.is_zero() {
        return Err(SubscriptionError::TargetIsZero);
    };
    let context = ContributorContext {
        next_pay_day: state.next_pay_day.u64(),
        block_time: env.block.time.seconds(),
        total_weight: state.total_weight.u128(),
        contributor_emissions: (state.emissions * (Decimal::one() - config.emission_user_share))
            .u128() as u64,
        payout_ratio: Decimal::from_ratio(state.expense, state.target),
        base_denom: config.base_denom,
        token_address: config.project_token.into_string(),
        proxy_address: base_state.proxy_address.into_string(),
    };

    let msgs = match contributor_addr {
        Some(contributor_addr) => {
            let compensation = CONTRIBUTORS.load(deps.storage, contributor_addr.as_bytes())?;
            // If no msgs just error
            let msg = process_contributor(
                contributor_addr.as_bytes(),
                deps.storage,
                compensation,
                &context,
            )?
            .unwrap();
            vec![msg]
        }
        None => CONTRIBUTORS.page_without_accumulator(
            deps.branch(),
            page_limit,
            &context,
            process_contributor,
        )?,
    };

    Ok(response
        .add_attribute("action:", "claim compensation")
        .add_attribute(
            "paging_complete",
            format!(
                "{}",
                CONTRIBUTORS
                    .load_status(deps.storage)?
                    .accumulator
                    .is_none()
            ),
        )
        .add_messages(msgs))
}

fn remove_contributor_from_storage(
    store: &mut dyn Storage,
    contributor_addr: &[u8],
) -> StdResult<()> {
    // Load all needed states
    let mut state = CON_STATE.load(store)?;

    let maybe_compensation = CONTRIBUTORS.may_load(store, contributor_addr)?;

    match maybe_compensation {
        Some(current_compensation) => {
            state.total_weight -= Uint128::from(current_compensation.weight);
            state.target = state
                .target
                .checked_sub(Uint64::from(current_compensation.base))?;
            // Can only get paid on pay day after next pay day
            CONTRIBUTORS.remove(store, contributor_addr)?;
            CON_STATE.save(store, &state)?;
        }
        None => {
            return Err(StdError::GenericErr {
                msg: "contributor is not registered".to_string(),
            })
        }
    };
    Ok(())
}

/// Checks if contributor has already claimed his share or if he's no longer eligible to claim a compensation.
fn process_contributor(
    contributor_key: &[u8],
    store: &mut dyn Storage,
    mut compensation: Compensation,
    context: &ContributorContext,
) -> StdResult<Option<CosmosMsg<Empty>>> {
    if compensation.next_pay_day.u64() > context.block_time {
        return Err(StdError::GenericErr {
            msg: "You cant claim before your next pay day.".to_string(),
        });
    } else if compensation.expiration.u64() < context.block_time {
        // remove contributor
        remove_contributor_from_storage(store, contributor_key)?;
        return Ok(None);
    }
    // update compensation details
    compensation.next_pay_day = context.next_pay_day.into();
    CONTRIBUTORS.unsafe_save(store, contributor_key, &compensation)?;

    let base_pay: Uint128 = Uint128::new(compensation.base as u128) * context.payout_ratio;

    let tokens = if context.total_weight != 0 {
        (Uint128::new(context.contributor_emissions as u128)
            * Decimal::from_ratio(compensation.weight as u128, context.total_weight))
        .u128()
    } else {
        0u128
    };

    pay_msg(
        base_pay.u128(),
        context.base_denom.clone(),
        tokens,
        context.token_address.clone(),
        String::from_utf8_lossy(contributor_key).to_string(),
        context.proxy_address.clone(),
    )
}

/// Constructs the proxy execute msgs for transferring funds
fn pay_msg(
    base_amount: u128,
    base_denom: String,
    token_amount: u128,
    token_addr: String,
    receiver: String,
    proxy_addr: String,
) -> StdResult<Option<CosmosMsg>> {
    let mut msgs = vec![];
    if base_amount != 0 {
        msgs.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: receiver.clone(),
            amount: vec![Coin {
                denom: base_denom,
                amount: Uint128::from(base_amount),
            }],
        }))
    }

    if token_amount != 0 {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: token_addr,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: receiver,
                amount: token_amount.into(),
            })?,
            funds: vec![],
        }))
    }
    if msgs.is_empty() {
        Ok(None)
    } else {
        Ok(Some(send_to_proxy(msgs, &Addr::unchecked(proxy_addr))?))
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
    payment_asset: Option<AssetInfo>,
    version_control_address: Option<String>,
    factory_address: Option<String>,
    subscription_cost: Option<Uint64>,
) -> SubscriptionResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config: SubscriptionConfig = SUB_CONFIG.load(deps.storage)?;

    if let Some(version_control_address) = version_control_address {
        // validate address format
        config.version_control_address = deps.api.addr_validate(&version_control_address)?;
    }

    if let Some(factory_address) = factory_address {
        // validate address format
        config.factory_address = deps.api.addr_validate(&factory_address)?;
    }

    if let Some(subscription_cost) = subscription_cost {
        // validate address format
        config.subscription_cost = subscription_cost;
    }

    if let Some(payment_asset) = payment_asset {
        config.payment_asset = payment_asset;
    }

    SUB_CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update subscriber config"))
}

// Only Admin can execute it
#[allow(clippy::too_many_arguments)]
pub fn update_contribution_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    protocol_income_share: Option<Decimal>,
    emission_user_share: Option<Decimal>,
    max_emissions_multiple: Option<Decimal>,
    project_token: Option<String>,
    emissions_amp_factor: Option<Uint128>,
    emissions_offset: Option<Uint128>,
    base_denom: Option<String>,
) -> SubscriptionResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut config = CON_CONFIG.load(deps.storage)?;

    if let Some(protocol_income_share) = protocol_income_share {
        // validate address format
        config.protocol_income_share = protocol_income_share;
    }

    if let Some(emission_user_share) = emission_user_share {
        // validate address format
        config.emission_user_share = emission_user_share;
    }

    if let Some(max_emissions_multiple) = max_emissions_multiple {
        // validate address format
        config.max_emissions_multiple = max_emissions_multiple;
    }

    if let Some(emissions_amp_factor) = emissions_amp_factor {
        // validate address format
        config.emissions_amp_factor = emissions_amp_factor;
    }

    if let Some(project_token) = project_token {
        // validate address format
        config.project_token = deps.api.addr_validate(&project_token)?;
    }

    if let Some(emissions_offset) = emissions_offset {
        // validate address format
        config.emissions_offset = emissions_offset;
    }

    if let Some(base_denom) = base_denom {
        // validate address format
        config.base_denom = base_denom;
    }

    CON_CONFIG.save(deps.storage, &config.verify()?)?;

    Ok(Response::new().add_attribute("action", "update contribution config"))
}

// fn update_income(
//     deps: DepsMut,
//     config: Config,
//     state: State,
//     subscription_state: subscription::state::State,
// ) -> _ {
//     let proxy_addr = BASESTATE.load(deps.storage)?.proxy_address;
//     let proxy_balance = config
//         .payment_asset
//         .query_balance(&deps.querier, proxy_addr)?;

//     if proxy_balance.u128() > state.target.u64() as u128 {
//         CON_STATE.update(deps.storage, |state| {
//             Ok(State {
//                 expense: state.target,
//                 ..state
//             })
//         })
//     }
// }
