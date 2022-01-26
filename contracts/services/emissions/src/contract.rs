#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::state::{Config, ALLOCATIONS, CONFIG, STATE};
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use pandora::emissions::msg::{
    AllocationInfo, AllocationResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
    ReceiveMsg, Schedule, SimulateWithdrawResponse, StateResponse,
};
use std::cmp;

//----------------------------------------------------------------------------------------
// Entry Points
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
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
            gov: deps.api.addr_validate(&msg.gov)?,
            refund_recepient: deps.api.addr_validate(&msg.refund_recepient)?,
            whale_token: deps.api.addr_validate(&msg.whale_token)?,
            default_unlock_schedule: msg.default_unlock_schedule,
        },
    )?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
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

#[cfg_attr(not(feature = "library"), entry_point)]
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
    if deposit_token != config.whale_token {
        return Err(StdError::generic_err("Only WHALE token can be deposited"));
    }

    // CHECK :: Number of WHALE Tokens sent need to be equal to the sum of newly vested balances
    if deposit_amount != allocations.iter().map(|params| params.1.total_amount).sum() {
        return Err(StdError::generic_err("WHALE deposit amount mismatch"));
    }

    state.total_whale_deposited += deposit_amount;
    state.remaining_whale_tokens += deposit_amount;

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

    let unlock_schedule = match &allocation.unlock_schedule {
        Some(schedule) => schedule,
        None => &config.default_unlock_schedule,
    };

    let whale_unlocked = compute_vested_or_unlocked_amount(
        env.block.time.seconds(),
        allocation.total_amount,
        unlock_schedule,
    );
    let whale_vested = compute_vested_or_unlocked_amount(
        env.block.time.seconds(),
        allocation.total_amount,
        &allocation.vest_schedule,
    );

    let whale_free = cmp::min(whale_vested, whale_unlocked);

    // Withdrawable amount
    let whale_withdrawable = whale_free - allocation.withdrawn_amount;

    // Check :: Is valid request ?
    if whale_withdrawable.is_zero() {
        return Err(StdError::generic_err("No unlocked WHALE to be withdrawn"));
    }

    // Init Response
    let mut response = Response::new().add_attribute("action", "withdraw");

    // UPDATE :: state & allocation
    allocation.withdrawn_amount += whale_withdrawable;
    state.remaining_whale_tokens -= whale_withdrawable;

    // SAVE :: state & allocation
    STATE.save(deps.storage, &state)?;
    ALLOCATIONS.save(deps.storage, &info.sender, &allocation)?;

    response = response
        .add_message(WasmMsg::Execute {
            contract_addr: config.whale_token.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: config.gov.to_string(),
                amount: whale_withdrawable,
            })?,
            funds: vec![],
        })
        .add_attribute("user", info.sender.to_string())
        .add_attribute("withdrawn_amount", whale_withdrawable.to_string());

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

    let unlock_schedule = match &allocation.unlock_schedule {
        Some(schedule) => schedule,
        None => &config.default_unlock_schedule,
    };

    let timestamp = env.block.time.seconds();
    let whale_unlocked =
        compute_vested_or_unlocked_amount(timestamp, allocation.total_amount, unlock_schedule);

    // Calculate WHALE tokens to be refunded
    let whale_to_refund = allocation.total_amount - whale_unlocked;

    if whale_to_refund.is_zero() {
        return Err(StdError::generic_err("No WHALE available to refund."));
    }

    // Set the total allocation amount to the current unlocked amount
    // This means user will not get any new tokens and the currently
    // unlocked tokens will vest based on the vesting schedule
    allocation.total_amount = whale_unlocked;
    allocation.unlock_schedule = None;

    // Update state
    state.total_whale_deposited -= whale_to_refund;
    state.remaining_whale_tokens -= whale_to_refund;

    // SAVE :: state & allocation
    STATE.save(deps.storage, &state)?;
    ALLOCATIONS.save(
        deps.storage,
        &deps.api.addr_validate(&user_address)?,
        &allocation,
    )?;

    let msgs: Vec<WasmMsg> = vec![WasmMsg::Execute {
        contract_addr: config.whale_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: config.refund_recepient.to_string(),
            amount: whale_to_refund,
        })?,
        funds: vec![],
    }];

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("user_address", user_address)
        .add_attribute("new_total_allocation", allocation.total_amount)
        .add_attribute("whale_refunded", whale_to_refund))
}

//----------------------------------------------------------------------------------------
// Handle Queries
//----------------------------------------------------------------------------------------

/// @dev Config Query
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        gov: config.gov.to_string(),
        owner: config.owner.to_string(),
        refund_recepient: config.refund_recepient.to_string(),
        whale_token: config.whale_token.to_string(),
        default_unlock_schedule: config.default_unlock_schedule,
    })
}

/// @dev State Query
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.may_load(deps.storage)?.unwrap_or_default();
    Ok(StateResponse {
        total_whale_deposited: state.total_whale_deposited,
        remaining_whale_tokens: state.remaining_whale_tokens,
    })
}

/// @dev Allocation Query
fn query_allocation(deps: Deps, _env: Env, account: String) -> StdResult<AllocationResponse> {
    let user_address = deps.api.addr_validate(&account)?;
    let allocation_info = ALLOCATIONS.load(deps.storage, &user_address)?;

    Ok(AllocationResponse {
        total_amount: allocation_info.total_amount,
        withdrawn_amount: allocation_info.withdrawn_amount,
        vest_schedule: allocation_info.vest_schedule,
        unlock_schedule: allocation_info.unlock_schedule,
    })
}

/// @dev Query function to fetch allocation state at any future timestamp
/// @params account : Account address whose allocation state is to be calculated
/// @params timestamp : Timestamp at which allocation state is to be calculated
fn query_simulate_withdraw(
    deps: Deps,
    env: Env,
    account: String,
    timestamp: Option<u64>,
) -> StdResult<SimulateWithdrawResponse> {
    let user_address = deps.api.addr_validate(&account)?;
    let allocation_info = ALLOCATIONS.load(deps.storage, &user_address)?;
    let config = CONFIG.load(deps.storage)?;

    let timestamp_ = match timestamp {
        Some(timestamp) => {
            if timestamp < env.block.time.seconds() {
                env.block.time.seconds()
            } else {
                timestamp
            }
        }
        None => env.block.time.seconds(),
    };

    Ok(compute_withdraw_amounts(
        timestamp_,
        &allocation_info,
        config.default_unlock_schedule,
    ))
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

    let whale_unlocked =
        compute_vested_or_unlocked_amount(timestamp, params.total_amount, unlock_schedule);
    let whale_vested =
        compute_vested_or_unlocked_amount(timestamp, params.total_amount, &params.vest_schedule);

    let mut free_whale = whale_vested;
    if timestamp < params.vest_schedule.start_time + params.vest_schedule.cliff {
        free_whale = Uint128::zero();
    }

    // Withdrawable amount is unlocked amount minus the amount already withdrawn
    let whale_withdrawable = free_whale - params.withdrawn_amount;

    SimulateWithdrawResponse {
        total_whale_locked: params.total_amount,
        total_whale_unlocked: whale_unlocked,
        total_whale_vested: whale_vested,
        withdrawn_amount: params.withdrawn_amount,
        withdrawable_amount: whale_withdrawable,
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
