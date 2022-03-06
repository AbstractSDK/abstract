use std::convert::TryInto;

use cosmwasm_std::{
    from_binary, Addr, Decimal, Deps, DepsMut, Env, Fraction, MessageInfo,
    Response, StdResult, Uint128, Uint64,
};
use cw20::{Cw20ReceiveMsg};
use pandora_os::core::treasury::msg::send_to_treasury;
use pandora_os::util::deposit_manager::Deposit;
use terraswap::asset::{Asset, AssetInfo};

use pandora_os::core::treasury::dapp_base::state::{ADMIN, BASESTATE};

use crate::contract::PaymentResult;
use crate::error::PaymentError;
use pandora_os::dapps::payout::{DepositHookMsg, Compensation};
use crate::state::{
    IncomeAccumulator, State, CLIENTS, CONFIG, CONTRIBUTORS, MONTH, STATE,
};

/// handler function invoked when the vault dapp contract receives
/// a transaction. In this case it is triggered when either a LP tokens received
/// by the contract or when the deposit asset is a cw20 asset.
pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> PaymentResult {
    match from_binary(&cw20_msg.msg)? {
        DepositHookMsg::Pay { os_id } => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Token {
                    contract_addr: msg_info.sender.to_string(),
                },
                amount: cw20_msg.amount,
            };
            try_pay(deps, msg_info, asset, Some(cw20_msg.sender), os_id)
        }
    }
}

/// Called when either paying with a native token or when paying
/// with a CW20.
pub fn try_pay(
    deps: DepsMut,
    msg_info: MessageInfo,
    asset: Asset,
    sender: Option<String>,
    os_id: u32,
) -> PaymentResult {
    // Load all needed states
    let config = CONFIG.load(deps.storage)?;
    let base_state = BASESTATE.load(deps.storage)?;
    // Get the liquidity provider address
    match sender {
        Some(addr) => Addr::unchecked(addr),
        None => {
            // Check if deposit matches claimed deposit.
            if asset.is_native_token() {
                // If native token, assert claimed amount is correct
                asset.assert_sent_native_token_balance(&msg_info)?;
                msg_info.sender
            } else {
                // Can't add liquidity with cw20 if not using the hook
                return Err(PaymentError::NotUsingCW20Hook {});
            }
        }
    };

    // Construct deposit info
    let deposit_info = config.payment_asset;

    // Assert payment asset and claimed asset infos are the same
    if deposit_info != asset.info {
        return Err(PaymentError::WrongToken {});
    }

    let mut customer_balance = CLIENTS.data.load(deps.storage, &os_id.to_be_bytes())?;
    customer_balance.increase((asset.amount.u128() as u64).into());

    CLIENTS
        .data
        .save(deps.storage, &os_id.to_be_bytes(), &customer_balance)?;

    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Deposit to payment module")),
        ("Received funds:", asset.to_string()),
    ];

    Ok(Response::new().add_attributes(attrs).add_message(
        // Send the received asset to the treasury
        asset.into_msg(&deps.querier, base_state.treasury_address)?,
    ))
}

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

    let maybe_compensation = CONTRIBUTORS.may_load(deps.storage, &contributor_addr)?;

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
            compensation.next_pay_day = state.next_pay_day + Uint64::from(MONTH);
        }
    };

    CONTRIBUTORS.save(deps.storage, &contributor_addr, &compensation)?;
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
    page_limit: Option<u32>,
) -> PaymentResult {
    let mut state: State = STATE.load(deps.storage)?;

    let mut response = Response::new();

    // Are we beyond the next pay time?
    if state.next_pay_day.u64() < env.block.time.seconds() {
        // First tally income, then set next block time
        tally_income(deps.branch(), env, page_limit)?;
        let info = CLIENTS.status.load(deps.storage)?;
        return Ok(Response::new().add_attributes(vec![
            ("Action:", String::from("Tally income")),
            ("Progress:", info.progress()),
        ]));
    }

    let mut compensation = CONTRIBUTORS.load(deps.storage, &info.sender.to_string())?;

    if compensation.next_pay_day.u64() > env.block.time.seconds() {
        return Err(PaymentError::WaitForNextPayday(
            compensation.next_pay_day.u64(),
        ));
    } else if compensation.expiration.u64() < env.block.time.seconds() {
        // remove contributor
        return remove_contributor_from_storage(deps, info.sender.to_string())
            .map(|_| Response::new());
    }
    // update compensation details
    compensation.next_pay_day = state.next_pay_day;
    CONTRIBUTORS.save(deps.storage, &info.sender.to_string(), &compensation)?;

    let config = CONFIG.load(deps.storage)?;
    let base_state = BASESTATE.load(deps.storage)?;

    // base amount payment
    let amount = Uint128::from(compensation.base) * state.expense_ratio;
    if !amount.is_zero() {
        let compensation_msg = send_to_treasury(
            vec![Asset {
                info: config.payment_asset,
                amount,
            }
            .into_msg(&deps.querier, info.sender.clone())?],
            &base_state.treasury_address,
        )?;
        response = response.add_message(compensation_msg);
    }

    let mint_income = Uint128::from(state.income.checked_sub(state.expense)?);
    let mint_price = Decimal::from_ratio(config.ratio * mint_income, Uint128::from(state.expense));
    let total_mints = mint_income * mint_price.inv().unwrap();

    // token emissions payment
    let amount = total_mints * Decimal::from_ratio(compensation.weight, state.total_weight);
    if !amount.is_zero() && state.token_cap > amount {
        state.token_cap -= amount;

        // Send tokens
        let token_msg = send_to_treasury(
            vec![Asset {
                info: AssetInfo::Token {
                    contract_addr: config.project_token.into(),
                },
                amount,
            }
            .into_msg(&deps.querier, info.sender.clone())?],
            &base_state.treasury_address,
        )?;
        response = response.add_message(token_msg);
    }

    Ok(response.add_attribute("Action:", "Claim compensation"))
}

fn tally_income(mut deps: DepsMut, env: Env, page_limit: Option<u32>) -> StdResult<()> {
    if let Some(res) = CLIENTS.page_with_accumulator(deps.branch(), page_limit, process_client)? {
        let state: State = STATE.load(deps.storage)?;
        let config = CONFIG.load(deps.storage)?;
        let max_expense: Uint128;
        let effective_spendable_amount = Uint128::from(res.income)
            - (Decimal::percent(100) - config.ratio) * Uint128::from(res.income);
        if effective_spendable_amount < state.target.into() {
            max_expense = effective_spendable_amount;
        } else {
            max_expense = state.target.into();
        }

        STATE.save(
            deps.storage,
            &State {
                income: Uint64::from(res.income),
                expense_ratio: Decimal::from_ratio(max_expense, state.target),
                expense: Uint64::from(max_expense.u128() as u64),
                next_pay_day: (env.block.time.seconds() + MONTH).into(),
                debtors: res.debtors,
                ..state
            },
        )?;
    }
    Ok(())
}

fn process_client(variables: (Vec<u8>, Deposit, Deps), acc: &mut IncomeAccumulator) -> () {
    let (key, mut deposit, deps) = variables;
    let os_id = u32::from_be_bytes(key.try_into().unwrap());
    let subscription_cost = CONFIG.load(deps.storage).unwrap().subscription_cost;

    match deposit.decrease(subscription_cost).ok() {
        Some(_) => {
            acc.income += subscription_cost.u64() as u32;
        }
        None => {
            acc.debtors.push(os_id);
        }
    }
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
