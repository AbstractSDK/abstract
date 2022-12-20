

use crate::queries::*;
use abstract_sdk::os::{
    vesting::{
        state::{Config, ALLOCATIONS, CONFIG, STATE},
        AllocationInfo, ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, Schedule,
        SimulateWithdrawResponse,
    },
    CW20_VESTING,
};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use std::cmp;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
//----------------------------------------------------------------------------------------
// Entry Points
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            refund_recipient: deps.api.addr_validate(&msg.refund_recipient)?,
            token: deps.api.addr_validate(&msg.token)?,
            default_unlock_schedule: msg.default_unlock_schedule,
        },
    )?;
    set_contract_version(deps.storage, CW20_VESTING, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => handle_receive_cw20(deps, env, info, cw20_msg),
        ExecuteMsg::Withdraw {} => handle_withdraw(deps, env, info),
        ExecuteMsg::Terminate { user_address } => handle_terminate(deps, env, info, user_address),
        ExecuteMsg::TransferOwnership { new_owner } => {
            handle_transfer_ownership(deps, env, info, new_owner)
        }
    }
}

fn handle_receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    match from_binary(&cw20_msg.msg)? {
        ReceiveMsg::CreateAllocations { allocations } => handle_create_allocations(
            deps,
            env,
            info.clone(),
            cw20_msg.sender,
            info.sender,
            cw20_msg.amount,
            allocations,
        ),
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::Allocation { account } => to_binary(&query_allocation(deps, env, account)?),
        QueryMsg::SimulateWithdraw { account, timestamp } => {
            to_binary(&query_simulate_withdraw(deps, env, account, timestamp)?)
        }
    }
}

//----------------------------------------------------------------------------------------
// Execute Points
//----------------------------------------------------------------------------------------

/// @dev Admin function to transfer contract ownership
/// @params new_owner : New admin
fn handle_transfer_ownership(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can call
    if info.sender != config.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    config.owner = deps.api.addr_validate(&new_owner)?;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

/// @dev Admin function to store new allocations
/// @params creator : User address who called this function
/// @params deposit_token : Token address being deposited
/// @params deposit_amount : Number of tokens sent along-with function call
/// @params allocations : Vector containing allocations  data
fn handle_create_allocations(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    creator: String,
    deposit_token: Addr,
    deposit_amount: Uint128,
    allocations: Vec<(String, AllocationInfo)>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.may_load(deps.storage)?.unwrap_or_default();

    // CHECK :: Only owner can create allocations
    if deps.api.addr_validate(&creator)? != config.owner {
        return Err(StdError::generic_err("Only owner can create allocations"));
    }

    // CHECK :: Only WHALE Token can be  can be deposited
    if deposit_token != config.token {
        return Err(StdError::generic_err("Only WHALE token can be deposited"));
    }

    // CHECK :: Number of WHALE Tokens sent need to be equal to the sum of newly vested balances
    if deposit_amount
        != allocations
            .iter()
            .map(|params| params.1.total_amount)
            .sum::<Uint128>()
    {
        return Err(StdError::generic_err("WHALE deposit amount mismatch"));
    }

    state.total_deposited += deposit_amount;
    state.remaining_tokens += deposit_amount;

    for allocation in allocations {
        let (user_unchecked, allocation_info) = allocation;
        let user = deps.api.addr_validate(&user_unchecked)?;

        match ALLOCATIONS.load(deps.storage, &user) {
            Ok(..) => {
                return Err(StdError::generic_err(format!(
                    "Allocation already exists for user {}",
                    user
                )));
            }
            Err(..) => match allocation_info.clone().unlock_schedule {
                Some(unlock_schedule) => {
                    if unlock_schedule.start_time + unlock_schedule.cliff
                        > allocation_info.vest_schedule.start_time
                            + allocation_info.vest_schedule.cliff
                    {
                        return Err(StdError::generic_err(format!("Invalid Allocation for {}. Unlock schedule needs to begin before vest schedule",user)));
                    }
                    ALLOCATIONS.save(deps.storage, &user, &allocation_info)?;
                }
                None => {
                    ALLOCATIONS.save(deps.storage, &user, &allocation_info)?;
                }
            },
        }
    }

    STATE.save(deps.storage, &state)?;
    Ok(Response::default())
}

/// @dev Facilitates withdrawal of unlocked WHALE Tokens
fn handle_withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.may_load(deps.storage)?.unwrap_or_default();
    let mut allocation = ALLOCATIONS.load(deps.storage, &info.sender)?;

    // Check :: Is valid request ?
    if allocation.total_amount == Uint128::zero()
        || allocation.total_amount == allocation.withdrawn_amount
    {
        return Err(StdError::generic_err("No unlocked WHALE to be withdrawn"));
    }

    // Check :: Are withdrawals allowed ?
    if env.block.time.seconds()
        < allocation.vest_schedule.start_time + allocation.vest_schedule.cliff
    {
        return Err(StdError::generic_err("Withdrawals not allowed yet"));
    }

    let unlock_schedule = match allocation.unlock_schedule.clone() {
        Some(schedule) => schedule,
        None => {
            if allocation.canceled {
                Schedule::zero()
            } else {
                config.default_unlock_schedule
            }
        }
    };

    let tokens_unlocked = compute_vested_or_unlocked_amount(
        env.block.time.seconds(),
        allocation.total_amount,
        &unlock_schedule,
    );
    let tokens_vested = compute_vested_or_unlocked_amount(
        env.block.time.seconds(),
        allocation.total_amount,
        &allocation.vest_schedule,
    );

    let tokens_free = cmp::min(tokens_vested, tokens_unlocked);

    // Withdrawable amount
    let tokens_withdrawable = tokens_free - allocation.withdrawn_amount;

    // Check :: Is valid request ?
    if tokens_withdrawable.is_zero() {
        return Err(StdError::generic_err("No unlocked WHALE to be withdrawn"));
    }

    // Init Response
    let mut response = Response::new().add_attribute("action", "withdraw");

    // UPDATE :: state & allocation
    allocation.withdrawn_amount += tokens_withdrawable;
    state.remaining_tokens -= tokens_withdrawable;

    // SAVE :: state & allocation
    STATE.save(deps.storage, &state)?;
    ALLOCATIONS.save(deps.storage, &info.sender, &allocation)?;

    response = response
        .add_message(WasmMsg::Execute {
            contract_addr: config.token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount: tokens_withdrawable,
            })?,
            funds: vec![],
        })
        .add_attribute("user", info.sender.to_string())
        .add_attribute("withdrawn_amount", tokens_withdrawable.to_string());

    Ok(response)
}

/// @dev Admin function to facilitate termination of the allocation schedule
/// @params user_address : User whose position is to be termintated
fn handle_terminate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    user_address: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // CHECK :: Only owner can call
    if info.sender != config.owner {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let mut state = STATE.may_load(deps.storage)?.unwrap_or_default();
    let mut allocation = ALLOCATIONS.load(deps.storage, &deps.api.addr_validate(&user_address)?)?;

    // Check if canceled
    if allocation.canceled {
        return Err(StdError::generic_err("Allocation is already canceled"));
    }

    let unlock_schedule = match &allocation.unlock_schedule {
        Some(schedule) => schedule,
        None => &config.default_unlock_schedule,
    };

    let timestamp = env.block.time.seconds();
    let tokens_unlocked =
        compute_vested_or_unlocked_amount(timestamp, allocation.total_amount, unlock_schedule);

    // Calculate WHALE tokens to be refunded
    let tokens_to_refund = allocation.total_amount - tokens_unlocked;

    if tokens_to_refund.is_zero() {
        return Err(StdError::generic_err("No WHALE available to refund."));
    }

    // Set the total allocation amount to the current unlocked amount
    // This means user will not get any new tokens and the currently
    // unlocked tokens will vest based on the vesting schedule
    allocation.total_amount = tokens_unlocked;
    allocation.unlock_schedule = None;
    allocation.canceled = true;

    // Update state
    state.total_deposited -= tokens_to_refund;
    state.remaining_tokens -= tokens_to_refund;

    // SAVE :: state & allocation
    STATE.save(deps.storage, &state)?;
    ALLOCATIONS.save(
        deps.storage,
        &deps.api.addr_validate(&user_address)?,
        &allocation,
    )?;

    let msgs: Vec<WasmMsg> = vec![WasmMsg::Execute {
        contract_addr: config.token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: config.refund_recipient.to_string(),
            amount: tokens_to_refund,
        })?,
        funds: vec![],
    }];

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("user_address", user_address)
        .add_attribute("new_total_allocation", allocation.total_amount)
        .add_attribute("tokens_refunded", tokens_to_refund))
}

//----------------------------------------------------------------------------------------
// Helper functions
//----------------------------------------------------------------------------------------

/// @dev Helper function. Calculates the current allocation state based on timestamp
/// @params timestamp : Timestamp for which simulation is to be made
/// @params params : Allocation schedule
/// @params default_unlock_schedule : Default unlock schedule
pub fn compute_withdraw_amounts(
    timestamp: u64,
    params: &AllocationInfo,
    default_unlock_schedule: Schedule,
) -> SimulateWithdrawResponse {
    let unlock_schedule = match &params.unlock_schedule {
        Some(schedule) => schedule,
        None => &default_unlock_schedule,
    };

    let tokens_unlocked =
        compute_vested_or_unlocked_amount(timestamp, params.total_amount, unlock_schedule);
    let tokens_vested =
        compute_vested_or_unlocked_amount(timestamp, params.total_amount, &params.vest_schedule);

    let mut free_whale = tokens_vested;
    if timestamp < params.vest_schedule.start_time + params.vest_schedule.cliff {
        free_whale = Uint128::zero();
    }

    // Withdrawable amount is unlocked amount minus the amount already withdrawn
    let tokens_withdrawable = free_whale - params.withdrawn_amount;

    SimulateWithdrawResponse {
        total_tokens_locked: params.total_amount,
        total_tokens_unlocked: tokens_unlocked,
        total_tokens_vested: tokens_vested,
        withdrawn_amount: params.withdrawn_amount,
        withdrawable_amount: tokens_withdrawable,
    }
}

/// @dev Helper function. Calculated unlocked / vested amount for an allocation
/// @params timestamp : Timestamp for which calculation is to be made
/// @params amount : Total number of tokens reserved for this allocation schedule
/// @params schedule : Allocation schedule
pub fn compute_vested_or_unlocked_amount(
    timestamp: u64,
    amount: Uint128,
    schedule: &Schedule,
) -> Uint128 {
    // Before the start time, no token will be vested/unlocked
    if timestamp < schedule.start_time {
        Uint128::zero()
    }
    // After the start_time,  tokens vest/unlock linearly between start time and end time
    else if timestamp < schedule.start_time + schedule.duration {
        amount.multiply_ratio(timestamp - schedule.start_time, schedule.duration)
    }
    // After end time, all tokens are fully vested/unlocked
    else {
        amount
    }
}
