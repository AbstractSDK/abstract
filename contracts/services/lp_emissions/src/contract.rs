use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use pandora::tokenomics::lp_emissions::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    StakerInfoResponse, StateResponse,
};

use crate::state::{Config, StakerInfo, State, CONFIG, STAKER_INFO, STATE};

//----------------------------------------------------------------------------------------
// Entry Points
//----------------------------------------------------------------------------------------

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
        whale_token: deps.api.addr_validate(&msg.whale_token)?,
        staking_token: deps.api.addr_validate(&msg.staking_token)?,
        staking_token_decimals: msg.staking_token_decimals,
        distribution_schedule: (0, 0, Uint128::zero()),
    };

    CONFIG.save(deps.storage, &config)?;

    STATE.save(
        deps.storage,
        &State {
            last_distributed: env.block.time.seconds(),
            total_bond_amount: Uint128::zero(),
            global_reward_index: Decimal::zero(),
            leftover: Uint128::zero(),
            reward_rate_per_token: Decimal::zero(),
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { new_owner } => update_config(deps, env, info, new_owner),
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::Unbond {
            amount,
            withdraw_pending_reward,
        } => unbond(deps, env, info, amount, withdraw_pending_reward),
        ExecuteMsg::Claim {} => try_claim(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State { timestamp } => to_binary(&query_state(deps, _env, timestamp)?),
        QueryMsg::StakerInfo { staker, timestamp } => {
            to_binary(&query_staker_info(deps, _env, staker, timestamp)?)
        }
        QueryMsg::Timestamp {} => to_binary(&query_timestamp(_env)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Err(StdError::generic_err("unimplemented"))
}
//----------------------------------------------------------------------------------------
// Handle Functions
//----------------------------------------------------------------------------------------

/// Only WHALE-UST LP Token can be sent to this contract via the Cw20ReceiveMsg Hook
/// @dev Increases user's staked LP Token balance via the Bond Function
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Bond {}) => {
            // only staking token contract can execute this message
            if config.staking_token != info.sender.as_str() {
                return Err(StdError::generic_err("unauthorized"));
            }
            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            bond(deps, env, cw20_sender, cw20_msg.amount)
        }
        Ok(Cw20HookMsg::UpdateRewardSchedule {
            period_start,
            period_finish,
            amount,
        }) => {
            // Only WHALE token contract can execute this message
            if config.whale_token != info.sender.as_str() {
                return Err(StdError::generic_err(
                    "Unauthorized : Only WHALE Token is allowed",
                ));
            }
            // Only owner can update the schedule
            if config.owner != cw20_msg.sender {
                return Err(StdError::generic_err("Only owner can update the schedule"));
            }
            update_reward_schedule(
                deps,
                env,
                info,
                period_start,
                period_finish,
                amount,
                cw20_msg.amount,
            )
        }

        Err(_) => Err(StdError::generic_err("data should be given")),
    }
}

/// @dev Called by receive_cw20(). Increases user's staked LP Token balance
/// @params sender_addr : User Address who sent the LP Tokens
/// @params amount : Number of LP Tokens transferred to the contract
pub fn bond(deps: DepsMut, env: Env, sender_addr: Addr, amount: Uint128) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info = STAKER_INFO
        .may_load(deps.storage, &sender_addr)?
        .unwrap_or_default();

    compute_reward(&config, &mut state, env.block.time.seconds());
    compute_staker_reward(&state, &mut staker_info)?;
    increase_bond_amount(&mut state, &mut staker_info, amount);

    // Store updated state with staker's staker_info
    STAKER_INFO.save(deps.storage, &sender_addr, &staker_info)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "Bond"),
        ("user", sender_addr.as_str()),
        ("amount", amount.to_string().as_str()),
        ("total_bonded", staker_info.bond_amount.to_string().as_str()),
    ]))
}

/// @dev Only owner can call this function. Updates the config
/// @params new_owner : New owner address
pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_owner: String,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // ONLY OWNER CAN UPDATE CONFIG
    if info.sender != config.owner {
        return Err(StdError::generic_err("Only owner can update configuration"));
    }

    // UPDATE :: ADDRESSES IF PROVIDED
    config.owner = deps.api.addr_validate(&new_owner)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "UpdateConfig")
        .add_attribute("new_owner", new_owner))
}

/// @dev Updates the reward schedule
pub fn update_reward_schedule(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    period_start: u64,
    period_finish: u64,
    amount_to_distribute: Uint128,
    amount_sent: Uint128,
) -> StdResult<Response> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;

    // Compute global reward
    compute_reward(&config, &mut state, env.block.time.seconds());

    // Invalid Period
    if period_start > period_finish {
        return Err(StdError::generic_err("Invalid Period"));
    }

    // contract must have enough tokens which can be used as incentives
    if amount_sent + state.leftover < amount_to_distribute {
        return Err(StdError::generic_err("insufficient funds on contract"));
    }

    // update distribution schedule (leftover is added to distribution amount)
    config.distribution_schedule = (period_start, period_finish, amount_to_distribute);

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "update_reward_schedule"),
        (
            "whale_to_distribute",
            amount_to_distribute.to_string().as_str(),
        ),
        (
            "total_bond_amount",
            state.total_bond_amount.to_string().as_str(),
        ),
    ]))
}

/// @dev Reduces user's staked position. WHALE Rewards are transferred along-with unstaked LP Tokens
/// @params amount :  Number of LP Tokens transferred to be unstaked
pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    withdraw_pending_reward: Option<bool>,
) -> StdResult<Response> {
    let sender_addr = info.sender;
    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info: StakerInfo = STAKER_INFO
        .may_load(deps.storage, &sender_addr)?
        .unwrap_or_default();

    if staker_info.bond_amount < amount {
        return Err(StdError::generic_err("Cannot unbond more than bond amount"));
    }

    compute_reward(&config, &mut state, env.block.time.seconds());
    compute_staker_reward(&state, &mut staker_info)?;
    decrease_bond_amount(&mut state, &mut staker_info, amount);
    let mut messages = vec![];
    let mut claimed_rewards = Uint128::zero();

    if let Some(withdraw_pending_reward) = withdraw_pending_reward {
        if withdraw_pending_reward {
            claimed_rewards = staker_info.pending_reward;
            if claimed_rewards > Uint128::zero() {
                staker_info.pending_reward = Uint128::zero();
                messages.push(build_send_cw20_token_msg(
                    sender_addr.clone(),
                    config.whale_token,
                    claimed_rewards,
                )?);
            }
        }
    }

    // Store Staker info, depends on the left bond amount
    STAKER_INFO.save(deps.storage, &sender_addr, &staker_info)?;
    STATE.save(deps.storage, &state)?;

    messages.push(build_send_cw20_token_msg(
        sender_addr.clone(),
        config.staking_token,
        amount,
    )?);

    // UNBOND STAKED TOKEN , TRANSFER $WHALE
    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "Unbond"),
        ("user", sender_addr.as_str()),
        ("amount", amount.to_string().as_str()),
        ("total_bonded", staker_info.bond_amount.to_string().as_str()),
        ("claimed_rewards", claimed_rewards.to_string().as_str()),
    ]))
}

/// @dev Function to claim accrued WHALE Rewards
pub fn try_claim(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let sender_addr = info.sender;
    let config: Config = CONFIG.load(deps.storage)?;
    let mut state: State = STATE.load(deps.storage)?;
    let mut staker_info = STAKER_INFO
        .may_load(deps.storage, &sender_addr)?
        .unwrap_or_default();

    // Compute global reward & staker reward
    compute_reward(&config, &mut state, env.block.time.seconds());
    compute_staker_reward(&state, &mut staker_info)?;

    let accrued_rewards = staker_info.pending_reward;
    staker_info.pending_reward = Uint128::zero();

    STAKER_INFO.save(deps.storage, &sender_addr, &staker_info)?; // Update Staker Info
    STATE.save(deps.storage, &state)?; // Store updated state

    let mut messages = vec![];

    if accrued_rewards == Uint128::zero() {
        return Err(StdError::generic_err("No rewards to claim"));
    } else {
        messages.push(build_send_cw20_token_msg(
            sender_addr.clone(),
            config.whale_token,
            accrued_rewards,
        )?);
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "Claim"),
        ("user", sender_addr.as_str()),
        ("claimed_rewards", accrued_rewards.to_string().as_str()),
    ]))
}

//----------------------------------------------------------------------------------------
// Query Functions
//----------------------------------------------------------------------------------------

/// @dev Returns the contract's configuration
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse {
        owner: config.owner.to_string(),
        whale_token: config.whale_token.to_string(),
        staking_token: config.staking_token.to_string(),
        distribution_schedule: config.distribution_schedule,
    })
}

/// @dev Returns the contract's simulated state at a certain timestamp
/// /// @param timestamp : Option parameter. Contract's Simulated state is retrieved if the timestamp is provided   
pub fn query_state(deps: Deps, env: Env, timestamp: Option<u64>) -> StdResult<StateResponse> {
    let mut state: State = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    match timestamp {
        Some(timestamp) => {
            compute_reward(
                &config,
                &mut state,
                std::cmp::max(timestamp, env.block.time.seconds()),
            );
        }
        None => {
            compute_reward(&config, &mut state, env.block.time.seconds());
        }
    }

    Ok(StateResponse {
        last_distributed: state.last_distributed,
        total_bond_amount: state.total_bond_amount,
        global_reward_index: state.global_reward_index,
        leftover: state.leftover,
        reward_rate_per_token: state.reward_rate_per_token,
    })
}

/// @dev Returns the User's simulated state at a certain timestamp
/// @param staker : User address whose state is to be retrieved
/// @param timestamp : Option parameter. User's Simulated state is retrieved if the timestamp is provided   
pub fn query_staker_info(
    deps: Deps,
    env: Env,
    staker: String,
    timestamp: Option<u64>,
) -> StdResult<StakerInfoResponse> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;
    let mut staker_info = STAKER_INFO
        .may_load(deps.storage, &deps.api.addr_validate(&staker)?)?
        .unwrap_or_default();

    match timestamp {
        Some(timestamp) => {
            compute_reward(
                &config,
                &mut state,
                std::cmp::max(timestamp, env.block.time.seconds()),
            );
        }
        None => {
            compute_reward(&config, &mut state, env.block.time.seconds());
        }
    }

    compute_staker_reward(&state, &mut staker_info)?;

    Ok(StakerInfoResponse {
        staker,
        reward_index: staker_info.reward_index,
        bond_amount: staker_info.bond_amount,
        pending_reward: staker_info.pending_reward,
    })
}

/// @dev Returns the current timestamp
pub fn query_timestamp(env: Env) -> StdResult<u64> {
    Ok(env.block.time.seconds())
}

//----------------------------------------------------------------------------------------
// Helper Functions
//----------------------------------------------------------------------------------------

/// @dev Increases total LP shares and user's staked LP shares by `amount`
fn increase_bond_amount(state: &mut State, staker_info: &mut StakerInfo, amount: Uint128) {
    state.total_bond_amount += amount;
    staker_info.bond_amount += amount;
}

/// @dev Decreases total LP shares and user's staked LP shares by `amount`
fn decrease_bond_amount(state: &mut State, staker_info: &mut StakerInfo, amount: Uint128) {
    staker_info.bond_amount -= amount;
    state.total_bond_amount -= amount;
}

/// @dev Updates State's leftover and reward_rate_per_token params
fn compute_state_extra(config: &Config, state: &mut State, timestamp: u64) {
    let s = config.distribution_schedule;

    // not started yet
    if timestamp <= s.0 {
        state.leftover = s.2;
        state.reward_rate_per_token = Decimal::zero();
    }
    // already finished
    else if timestamp >= s.1 {
        state.leftover = Uint128::zero();
        state.reward_rate_per_token = Decimal::zero();
    }
    // s.0 < timestamp < s.1
    else {
        let duration = s.1 - s.0;
        let distribution_rate: Decimal = Decimal::from_ratio(s.2, duration);
        let time_left = s.1 - timestamp;
        state.leftover = distribution_rate * Uint128::from(time_left as u128);
        if state.total_bond_amount.is_zero() {
            state.reward_rate_per_token = Decimal::zero();
        } else {
            let denom = Uint128::from(10u128.pow(config.staking_token_decimals as u32));
            state.reward_rate_per_token =
                Decimal::from_ratio(distribution_rate * denom, state.total_bond_amount);
        }
    }
}

// compute distributed rewards and update global reward index
fn compute_reward(config: &Config, state: &mut State, timestamp: u64) {
    compute_state_extra(config, state, timestamp);

    if state.total_bond_amount.is_zero() {
        state.last_distributed = timestamp;
        return;
    }

    let mut distributed_amount: Uint128 = Uint128::zero();
    let s = config.distribution_schedule;
    if s.0 < timestamp && s.1 > state.last_distributed {
        let time_passed =
            std::cmp::min(s.1, timestamp) - std::cmp::max(s.0, state.last_distributed);
        let duration = s.1 - s.0;
        let distribution_rate: Decimal = Decimal::from_ratio(s.2, duration);
        distributed_amount += distribution_rate * Uint128::from(time_passed as u128);
    }

    state.last_distributed = timestamp;
    state.global_reward_index = state.global_reward_index
        + Decimal::from_ratio(distributed_amount, state.total_bond_amount);
}

/// @dev Computes user's accrued rewards
fn compute_staker_reward(state: &State, staker_info: &mut StakerInfo) -> StdResult<()> {
    let pending_reward = (staker_info.bond_amount * state.global_reward_index)
        - (staker_info.bond_amount * staker_info.reward_index);
    staker_info.reward_index = state.global_reward_index;
    staker_info.pending_reward += pending_reward;
    Ok(())
}

/// @dev Helper function to build `CosmosMsg` to send cw20 tokens to a recepient address
fn build_send_cw20_token_msg(
    recipient: Addr,
    token_contract_address: Addr,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_contract_address.into(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.into(),
            amount,
        })?,
        funds: vec![],
    }))
}
