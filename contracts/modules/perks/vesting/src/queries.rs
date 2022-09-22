use abstract_os::vesting::{
    state::{ALLOCATIONS, CONFIG, STATE},
    AllocationResponse, ConfigResponse, SimulateWithdrawResponse, StateResponse,
};
use cosmwasm_std::{Deps, Env, StdResult};

use crate::contract::compute_withdraw_amounts;

/// @dev Config Query
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        refund_recipient: config.refund_recipient.to_string(),
        token: config.token.to_string(),
        default_unlock_schedule: config.default_unlock_schedule,
    })
}

/// @dev State Query
pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.may_load(deps.storage)?.unwrap_or_default();
    Ok(StateResponse {
        total_deposited: state.total_deposited,
        remaining_tokens: state.remaining_tokens,
    })
}

/// @dev Allocation Query
pub fn query_allocation(deps: Deps, _env: Env, account: String) -> StdResult<AllocationResponse> {
    let user_address = deps.api.addr_validate(&account)?;
    let allocation_info = ALLOCATIONS.load(deps.storage, &user_address)?;

    Ok(AllocationResponse {
        total_amount: allocation_info.total_amount,
        withdrawn_amount: allocation_info.withdrawn_amount,
        vest_schedule: allocation_info.vest_schedule,
        unlock_schedule: allocation_info.unlock_schedule,
        canceled: allocation_info.canceled,
    })
}

/// @dev Query function to fetch allocation state at any future timestamp
/// @params account : Account address whose allocation state is to be calculated
/// @params timestamp : Timestamp at which allocation state is to be calculated
pub fn query_simulate_withdraw(
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
