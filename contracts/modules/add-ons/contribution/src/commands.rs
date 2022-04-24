use std::convert::TryInto;

use cosmwasm_std::{
    from_binary, Addr, Decimal, Deps, DepsMut, Env, Fraction, MessageInfo, Response, StdResult,
    Uint128, Uint64, CosmosMsg, WasmMsg, to_binary, Storage,
};
use cw20::Cw20ReceiveMsg;
use cw_asset::{Asset, AssetInfo};
use pandora_os::core::proxy::msg::send_to_proxy;
use pandora_os::modules::add_ons::subscription;
use pandora_os::util::deposit_manager::Deposit;

use pandora_os::modules::dapp_base::state::{ADMIN, BASESTATE};

use crate::contract::PaymentResult;
use crate::error::PaymentError;
use crate::state::{ State,  CONFIG, CONTRIBUTORS,  STATE, Config};
use pandora_os::modules::add_ons::contribution::{Compensation};

pub const MONTH: u64 = 60 * 60 * 24 * 30;

/// Function that adds/updates the contributor config of a given address
pub fn update_contributor(
    deps: DepsMut,
    msg_info: MessageInfo,
    contributor_addr: String,
    mut compensation: Compensation,
) -> PaymentResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    // Load all needed states
    let mut state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let maybe_compensation = CONTRIBUTORS.may_load(deps.storage, &contributor_addr.as_bytes())?;

    match maybe_compensation {
        Some(current_compensation) => {
            let weight_diff: i32 = current_compensation.weight as i32 - compensation.weight as i32;
            let base_diff: i32 = current_compensation.base as i32 - compensation.base as i32;
            state.total_weight =
                Uint128::from((state.total_weight.u128() as i128 + weight_diff as i128) as u128);
            state.target = Uint64::from((state.target.u64() as i64 + base_diff as i64) as u64);
        }
        None => {
            state.total_weight += Uint128::from(compensation.weight);
            state.target += Uint64::from(compensation.base);
            // Can only get paid on pay day after next pay day
            let next_pay_day = subscription::state::STATE.query(&deps.querier, config.subscription_contract)?.next_pay_day;
            compensation.next_pay_day = next_pay_day + Uint64::from(MONTH);
        }
    };

    CONTRIBUTORS.save(deps.storage, contributor_addr.as_bytes(), &compensation)?;
    STATE.save(deps.storage, &state)?;

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
) -> PaymentResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    remove_contributor_from_storage(deps, contributor_addr.clone())?;
    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Remove Contributor")),
        ("Address:", contributor_addr),
    ];

    Ok(Response::new().add_attributes(attrs))
}

pub fn try_claim(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contributor_addr: Option<String>,
    page_limit: Option<u32>,
) -> PaymentResult {
    let mut state: State = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let subscription_state = subscription::state::STATE.query(&deps.querier, config.subscription_contract)?;

    let response = Response::new();

    // First update subscription contract if needed
    if subscription_state.next_pay_day.u64() < env.block.time.seconds() && subscription_state.debtors.is_empty() {
        return update_subscription_state(&config.subscription_contract, page_limit)
    } else if subscription_state.next_pay_day != state.next_pay_day {
        update_income(deps, &mut state,subscription_state);
        state.next_pay_day = subscription_state.next_pay_day;
    }

    match contributor_addr {
        Some(contributor_addr) => {
            check_contributor_compensation(deps, env, contributor_addr, subscription_state.next_pay_day)?;
        },
        None => {
            CONTRIBUTORS.data.query(querier, remote_contract, k)
        },
    }

    let base_state = BASESTATE.load(deps.storage)?;

    // base amount payment
    let amount = Uint128::from(compensation.base) * state.expense_ratio;
    if !amount.is_zero() {
        let compensation_msg = send_to_proxy(
            vec![Asset {
                info: config.payment_asset,
                amount,
            }
            .transfer_msg(info.sender.clone())?],
            &base_state.proxy_address,
        )?;
        response = response.add_message(compensation_msg);
    }

    let mint_income = Uint128::from(state.income.checked_sub(state.expense)?);
    let mint_price = Decimal::from_ratio(config.ratio * mint_income, Uint128::from(state.expense));
    let total_mints = mint_income * mint_price.inv().unwrap();

    // token emissions payment
    let amount = total_mints * Decimal::from_ratio(compensation.weight, state.total_weight);
    if !amount.is_zero() {
        // Send tokens
        let token_msg = send_to_proxy(
            vec![Asset {
                info: AssetInfo::Cw20(config.project_token),
                amount,
            }
            .transfer_msg(info.sender)?],
            &base_state.proxy_address,
        )?;
        response = response.add_message(token_msg);
    }

    Ok(response.add_attribute("Action:", "Claim compensation"))
}

fn update_income(deps: DepsMut, config: Config, state: State,subscription_state: subscription::state::State) -> _ {
    let proxy_addr = BASESTATE.load(deps.storage)?.proxy_address;
    let proxy_balance = config.payment_asset.query_balance(&deps.querier, proxy_addr)?;

    if proxy_balance.u128() > state.target.u64() as u128 {
        STATE.update(deps.storage, |state| Ok(State {
            expense: state.target,
            .. state
        }))
    }

}

fn update_subscription_state(subscription_contract_addr: &Addr, page_limit: Option<u32>) -> Result<Response, PaymentError> {
    Ok(Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: subscription_contract_addr.to_string(),
        msg: to_binary(&subscription::msg::ExecuteMsg::PurgeDebtors { page_limit })?,
        funds: vec![],
    })))
}

fn remove_contributor_from_storage(
    deps: DepsMut,
    contributor_addr: String,
) -> Result<(), PaymentError> {
    // Load all needed states
    let mut state = STATE.load(deps.storage)?;

    let maybe_compensation = CONTRIBUTORS.may_load(deps.storage, &contributor_addr)?;

    match maybe_compensation {
        Some(current_compensation) => {
            state.total_weight -= Uint128::from(current_compensation.weight);
            state.target = state
                .target
                .checked_sub(Uint64::from(current_compensation.base))?;
            // Can only get paid on pay day after next pay day
            CONTRIBUTORS.remove(deps.storage, &contributor_addr);
            STATE.save(deps.storage, &state)?;
        }
        None => return Err(PaymentError::ContributorNotRegistered {}),
    };
    Ok(())
}

fn check_contributor_compensation(deps: DepsMut, env: Env, contributor_addr: String, next_pay_day: Uint64) -> Result<(), PaymentError> {
    let mut compensation = CONTRIBUTORS.load(deps.storage, &contributor_addr)?;
    
    if compensation.next_pay_day.u64() > env.block.time.seconds() {
        return Err(PaymentError::WaitForNextPayday(
            compensation.next_pay_day.u64(),
        ));
    } else if compensation.expiration.u64() < env.block.time.seconds() {
        // remove contributor
        return remove_contributor_from_storage(deps, contributor_addr);
    }
    // update compensation details
    compensation.next_pay_day = next_pay_day;
    CONTRIBUTORS.save(deps.storage, &contributor_addr, &compensation)?;

    Ok(())
}
