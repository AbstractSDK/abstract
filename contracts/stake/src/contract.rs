use std::collections::HashMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure_eq, from_slice, to_binary, Addr, Binary, Decimal, Deps, DepsMut, Empty, Env,
    MessageInfo, Order, Response, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_controllers::Claim;
use cw_storage_plus::Map;
use wyndex::asset::{addr_opt_validate, AssetInfo, AssetInfoValidated};
use wyndex::common::validate_addresses;
use wyndex::lp_converter::ExecuteMsg as ConverterExecuteMsg;
use wyndex::stake::{FundingInfo, InstantiateMsg, ReceiveMsg, UnbondingPeriod};

use crate::distribution::{
    apply_points_correction, execute_delegate_withdrawal, execute_distribute_rewards,
    execute_withdraw_rewards, query_delegated, query_distributed_rewards, query_distribution_data,
    query_undistributed_rewards, query_withdraw_adjustment_data, query_withdrawable_rewards,
};
use crate::utils::{create_undelegate_msg, CurveExt};
use cw2::set_contract_version;
use cw_utils::{ensure_from_older_version, maybe_addr, Expiration};

use crate::error::ContractError;
use crate::msg::{
    AllStakedResponse, AnnualizedReward, AnnualizedRewardsResponse, BondingInfoResponse,
    BondingPeriodInfo, ExecuteMsg, MigrateMsg, QueryMsg, RewardsPowerResponse, StakedResponse,
    TotalStakedResponse, TotalUnbondingResponse, UnbondAllResponse,
};
use crate::state::{
    Config, ConverterConfig, Distribution, TokenInfo, TotalStake, ADMIN, CLAIMS, CONFIG,
    DISTRIBUTION, REWARD_CURVE, STAKE, TOTAL_PER_PERIOD, TOTAL_STAKED, UNBOND_ALL,
};
use wynd_curve_utils::Curve;

const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;

// version info for migration info
const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_CRATE_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mut msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let api = deps.api;
    // Set the admin if provided
    ADMIN.set(deps.branch(), maybe_addr(api, msg.admin.clone())?)?;

    // min_bond is at least 1, so 0 stake -> non-membership
    let min_bond = std::cmp::max(msg.min_bond, Uint128::new(1));

    TOTAL_STAKED.save(deps.storage, &TokenInfo::default())?;

    // make sure they are sorted, this is important because the rest of the contract assumes the same
    // order everywhere and uses binary search in some places.
    msg.unbonding_periods.sort_unstable();

    // initialize total stake
    TOTAL_PER_PERIOD.save(
        deps.storage,
        &msg.unbonding_periods
            .iter()
            .map(|unbonding_period| (*unbonding_period, TotalStake::default()))
            .collect(),
    )?;

    // Initialize unbond all flag.
    UNBOND_ALL.save(deps.storage, &false)?;

    let config = Config {
        instantiator: info.sender,
        cw20_contract: deps.api.addr_validate(&msg.cw20_contract)?,
        tokens_per_power: msg.tokens_per_power,
        min_bond,
        unbonding_periods: msg.unbonding_periods,
        max_distributions: msg.max_distributions,
        unbonder: addr_opt_validate(deps.api, &msg.unbonder)?,
        converter: msg
            .converter
            .map(|conv| -> StdResult<ConverterConfig> {
                Ok(ConverterConfig {
                    contract: deps.api.addr_validate(&conv.contract)?,
                    pair_to: deps.api.addr_validate(&conv.pair_to)?,
                })
            })
            .transpose()?,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => {
            Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, admin)?)?)
        }
        ExecuteMsg::CreateDistributionFlow {
            manager,
            asset,
            rewards,
        } => execute_create_distribution_flow(deps, info, manager, asset, rewards),
        ExecuteMsg::Rebond {
            tokens,
            bond_from,
            bond_to,
        } => execute_rebond(deps, env, info, tokens, bond_from, bond_to),
        ExecuteMsg::Unbond {
            tokens: amount,
            unbonding_period,
        } => execute_unbond(deps, env, info, amount, unbonding_period),
        ExecuteMsg::QuickUnbond { stakers } => execute_quick_unbond(deps, env, info, stakers),
        ExecuteMsg::UnbondAll {} => execute_unbond_all(deps, info),
        ExecuteMsg::StopUnbondAll {} => execute_stop_unbond_all(deps, info),
        ExecuteMsg::Claim {} => execute_claim(deps, env, info),
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        ExecuteMsg::DistributeRewards { sender } => {
            execute_distribute_rewards(deps, env, info, sender)
        }
        ExecuteMsg::WithdrawRewards { owner, receiver } => {
            execute_withdraw_rewards(deps, info, owner, receiver)
        }
        ExecuteMsg::DelegateWithdrawal { delegated } => {
            execute_delegate_withdrawal(deps, info, delegated)
        }
        ExecuteMsg::FundDistribution { funding_info } => {
            execute_fund_distribution(env, deps, info, funding_info)
        }
        ExecuteMsg::MigrateStake {
            amount,
            unbonding_period,
        } => execute_migrate_stake(deps, env, info, amount, unbonding_period),
    }
}

/// Fund a previously created distribution flow with the given amount of native tokens.
/// Allows for providing multiple native tokens at once to update multiple distribution flows with the same optionally provided Curve.
pub fn execute_fund_distribution(
    env: Env,
    deps: DepsMut,
    info: MessageInfo,
    funding_info: FundingInfo,
) -> Result<Response, ContractError> {
    if UNBOND_ALL.load(deps.storage)? {
        return Err(ContractError::CannotDistributeIfUnbondAll {
            what: "funds".into(),
        });
    }

    if funding_info.start_time < env.block.time.seconds() {
        return Err(ContractError::PastStartingTime {});
    }

    let api = deps.api;
    let storage = deps.storage;

    for fund in info.funds {
        let asset = AssetInfo::Native(fund.denom);
        let validated_asset = asset.validate(api)?;
        update_reward_config(storage, validated_asset, fund.amount, funding_info.clone())?;
    }
    Ok(Response::default())
}

/// Triggers moving the stake from this staking contract to another staking contract
pub fn execute_migrate_stake(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    unbonding_period: u64,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let converter = cfg
        .converter
        .as_ref()
        .ok_or(ContractError::NoConverter {})?;

    remove_stake_without_total(
        deps.branch(),
        &env,
        &cfg,
        &info.sender,
        unbonding_period,
        amount,
    )?;

    // update total
    TOTAL_STAKED.update::<_, StdError>(deps.storage, |token_info| {
        Ok(TokenInfo {
            staked: token_info.staked.saturating_sub(amount),
            unbonding: token_info.unbonding,
        })
    })?;

    // directly send the tokens to the converter instead of providing claim
    Ok(Response::new()
        // send the tokens to the converter
        .add_message(WasmMsg::Execute {
            contract_addr: cfg.cw20_contract.into_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: converter.contract.to_string(),
                amount,
            })?,
            funds: vec![],
        })
        // once the tokens are transfered to the converter, we convert them
        .add_message(WasmMsg::Execute {
            contract_addr: converter.contract.to_string(),
            msg: to_binary(&ConverterExecuteMsg::Convert {
                sender: info.sender.to_string(),
                amount,
                unbonding_period,
                pair_contract_from: cfg.instantiator.into_string(),
                pair_contract_to: converter.pair_to.to_string(),
            })?,
            funds: vec![],
        })
        .add_attribute("action", "unbond")
        .add_attribute("amount", amount)
        .add_attribute("sender", info.sender))
}

/// Update reward config for the given asset with an additional amount of funding
fn update_reward_config(
    storage: &mut dyn Storage,
    validated_asset: AssetInfoValidated,
    sent_amount: Uint128,
    FundingInfo {
        start_time,
        distribution_duration,
        amount,
    }: FundingInfo,
) -> Result<(), ContractError> {
    // How can we validate the amount and curve? Monotonic decreasing check is below, given this is there still a need to test the amount?
    let previous_reward_curve = REWARD_CURVE.load(storage, &validated_asset)?;

    let end_time = start_time + distribution_duration;
    let schedule = Curve::saturating_linear((start_time, amount.u128()), (end_time, 0));

    let (min, max) = schedule.range();
    // Validate the the curve locks at most the amount provided and also fully unlocks all rewards sent
    if min != 0 || max > sent_amount.u128() {
        return Err(ContractError::InvalidRewards {});
    }

    // combine the two curves
    let new_reward_curve = previous_reward_curve.combine(&schedule);
    new_reward_curve.validate_monotonic_decreasing()?;

    REWARD_CURVE.save(storage, &validated_asset, &new_reward_curve)?;
    Ok(())
}

/// Create a new rewards distribution flow for the given asset as a reward
pub fn execute_create_distribution_flow(
    deps: DepsMut,
    info: MessageInfo,
    manager: String,
    asset: AssetInfo,
    rewards: Vec<(UnbondingPeriod, Decimal)>,
) -> Result<Response, ContractError> {
    // only admin can create distribution flow
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    // input validation
    let asset = asset.validate(deps.api)?;
    let manager = deps.api.addr_validate(&manager)?;

    // make sure the asset is not the staked token, since we distribute this contract's balance
    // and we definitely do not want to distribute the staked tokens.
    let config = CONFIG.load(deps.storage)?;
    if let AssetInfoValidated::Token(addr) = &asset {
        if addr == config.cw20_contract {
            return Err(ContractError::InvalidAsset {});
        }
    }

    // validate rewards unbonding periods
    if rewards
        .iter()
        .map(|(period, _)| period)
        .ne(config.unbonding_periods.iter())
    {
        return Err(ContractError::InvalidRewards {});
    }
    // make sure rewards are monotonically increasing (equality is allowed)
    // this assumes that `config.unbonding_periods` (and therefore also `rewards`) is sorted (checked in instantiate)
    if rewards.windows(2).any(|w| w[0].1 > w[1].1) {
        return Err(ContractError::InvalidRewards {});
    }

    // make sure to respect the distribution count limit to create an upper bound for all the staking operations
    let keys = DISTRIBUTION
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    if keys.len() >= (config.max_distributions as usize) {
        return Err(ContractError::TooManyDistributions(
            config.max_distributions,
        ));
    }

    // make sure the distribution does not exist already
    if keys.contains(&asset) {
        return Err(ContractError::DistributionAlreadyExists(asset));
    }

    REWARD_CURVE.save(deps.storage, &asset, &Curve::constant(0))?;

    DISTRIBUTION.save(
        deps.storage,
        &asset,
        &Distribution {
            manager,
            reward_multipliers: rewards,
            shares_per_point: Uint128::zero(),
            shares_leftover: 0,
            distributed_total: Uint128::zero(),
            withdrawable_total: Uint128::zero(),
        },
    )?;

    Ok(Response::default())
}

pub fn execute_rebond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    bond_from: u64,
    bond_to: u64,
) -> Result<Response, ContractError> {
    if UNBOND_ALL.load(deps.storage)? {
        return Err(ContractError::CannotRebondIfUnbondAll {});
    }

    // Raise if no amount was provided
    if amount == Uint128::zero() {
        return Err(ContractError::NoRebondAmount {});
    }
    // Short out with an error if trying to rebond to itself
    if bond_from == bond_to {
        return Err(ContractError::SameUnbondingRebond {});
    }

    let cfg = CONFIG.load(deps.storage)?;

    if cfg.unbonding_periods.binary_search(&bond_from).is_err() {
        return Err(ContractError::NoUnbondingPeriodFound(bond_from));
    }
    if cfg.unbonding_periods.binary_search(&bond_to).is_err() {
        return Err(ContractError::NoUnbondingPeriodFound(bond_to));
    }

    let distributions: Vec<_> = DISTRIBUTION
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    // calculate rewards power before updating the stake
    let old_rewards = calc_rewards_powers(deps.storage, &cfg, &info.sender, distributions.iter())?;

    // Reduce the bond_from
    let mut old_stake_from = Uint128::zero();
    let new_stake_from = STAKE
        .update(
            deps.storage,
            (&info.sender, bond_from),
            |bonding_info| -> StdResult<_> {
                let mut bonding_info = bonding_info.unwrap_or_default();
                old_stake_from = bonding_info.total_stake();
                // Release the stake, also accounting for locked tokens, raising if there is not enough tokens
                bonding_info.release_stake(&env, amount)?;
                Ok(bonding_info)
            },
        )?
        .total_stake();

    // Increase the bond_to
    let mut old_stake_to = Uint128::zero();
    let new_stake_to = STAKE
        .update(
            deps.storage,
            (&info.sender, bond_to),
            |bonding_info| -> StdResult<_> {
                let mut bonding_info = bonding_info.unwrap_or_default();
                old_stake_to = bonding_info.total_stake();

                if bond_from > bond_to {
                    bonding_info.add_locked_tokens(
                        env.block.time.plus_seconds(bond_from - bond_to),
                        amount,
                    );
                } else {
                    bonding_info.add_unlocked_tokens(amount);
                };
                Ok(bonding_info)
            },
        )?
        .total_stake();

    update_total_stake(
        deps.storage,
        &cfg,
        bond_from,
        old_stake_from,
        new_stake_from,
    )?;
    update_total_stake(deps.storage, &cfg, bond_to, old_stake_to, new_stake_to)?;

    // update the adjustment data for all distributions
    for ((asset_info, mut distribution), old_reward_power) in
        distributions.into_iter().zip(old_rewards.into_iter())
    {
        let new_reward_power = distribution.calc_rewards_power(deps.storage, &cfg, &info.sender)?;
        update_rewards(
            deps.storage,
            &asset_info,
            &info.sender,
            &mut distribution,
            old_reward_power,
            new_reward_power,
        )?;

        // save updated distribution
        DISTRIBUTION.save(deps.storage, &asset_info, &distribution)?;
    }

    Ok(Response::new()
        .add_attribute("action", "rebond")
        .add_attribute("amount", amount)
        .add_attribute("bond_from", bond_from.to_string())
        .add_attribute("bond_to", bond_to.to_string()))
}

pub fn execute_bond(
    deps: DepsMut,
    env: Env,
    sender_cw20_contract: Addr,
    amount: Uint128,
    unbonding_period: u64,
    sender: Addr,
) -> Result<Response, ContractError> {
    let delegations = vec![(sender.to_string(), amount)];
    let res = execute_mass_bond(
        deps,
        env,
        sender_cw20_contract,
        amount,
        unbonding_period,
        delegations,
    )?;
    Ok(res.add_attribute("sender", sender))
}

pub fn execute_mass_bond(
    deps: DepsMut,
    _env: Env,
    sender_cw20_contract: Addr,
    amount_sent: Uint128,
    unbonding_period: u64,
    delegate_to: Vec<(String, Uint128)>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // ensure that cw20 token contract's addresses matches
    if cfg.cw20_contract != sender_cw20_contract {
        return Err(ContractError::Cw20AddressesNotMatch {
            got: sender_cw20_contract.into(),
            expected: cfg.cw20_contract.into(),
        });
    }

    if cfg
        .unbonding_periods
        .binary_search(&unbonding_period)
        .is_err()
    {
        return Err(ContractError::NoUnbondingPeriodFound(unbonding_period));
    }

    // ensure total is <= amount sent
    let total = delegate_to.iter().map(|(_, x)| x).sum();
    if total > amount_sent {
        return Err(ContractError::MassDelegateTooMuch { total, amount_sent });
    }

    // update this for every user
    let mut distributions: Vec<_> = DISTRIBUTION
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    // loop over all delegates, adding to their stake
    for (sender, amount) in delegate_to {
        let sender = deps.api.addr_validate(&sender)?;

        // calculate rewards power before updating the stake
        let old_rewards = calc_rewards_powers(deps.storage, &cfg, &sender, distributions.iter())?;

        // add to the sender's stake
        let mut old_stake = Uint128::zero();
        let new_stake = STAKE
            .update(
                deps.storage,
                (&sender, unbonding_period),
                |bonding_info| -> StdResult<_> {
                    let mut bonding_info = bonding_info.unwrap_or_default();
                    old_stake = bonding_info.total_stake();
                    bonding_info.add_unlocked_tokens(amount);
                    Ok(bonding_info)
                },
            )?
            .total_stake();

        update_total_stake(deps.storage, &cfg, unbonding_period, old_stake, new_stake)?;

        // update the adjustment data for all distributions
        distributions = distributions
            .into_iter()
            .zip(old_rewards.into_iter())
            .map(|((asset_info, mut distribution), old_reward_power)| {
                let new_reward_power =
                    distribution.calc_rewards_power(deps.storage, &cfg, &sender)?;
                update_rewards(
                    deps.storage,
                    &asset_info,
                    &sender,
                    &mut distribution,
                    old_reward_power,
                    new_reward_power,
                )?;
                Ok((asset_info, distribution))
            })
            .collect::<StdResult<Vec<_>>>()?;
    }

    // save all distributions (now updated)
    for (asset_info, distribution) in distributions {
        DISTRIBUTION.save(deps.storage, &asset_info, &distribution)?;
    }

    // update total after all individuals are handled
    TOTAL_STAKED.update::<_, StdError>(deps.storage, |token_info| {
        Ok(TokenInfo {
            staked: token_info.staked + amount_sent,
            unbonding: token_info.unbonding,
        })
    })?;

    Ok(Response::new()
        .add_attribute("action", "bond")
        .add_attribute("amount", amount_sent))
}

/// Updates the total stake for the given unbonding period
/// Make sure to always pass in the full old and new stake of one staker for the given unbonding period
fn update_total_stake(
    storage: &mut dyn Storage,
    cfg: &Config,
    unbonding_period: UnbondingPeriod,
    old_stake: Uint128,
    new_stake: Uint128,
) -> Result<(), ContractError> {
    // get current total stakes
    let mut totals = TOTAL_PER_PERIOD.load(storage)?;
    let total_idx = totals
        .binary_search_by(|(period, _)| period.cmp(&unbonding_period))
        .map_err(|_| ContractError::NoUnbondingPeriodFound(unbonding_period))?;
    let total = &mut totals[total_idx].1;

    // update the total amount staked in this unbonding period
    total.staked = if old_stake <= new_stake {
        total.staked.checked_add(new_stake - old_stake)?
    } else {
        total.staked.checked_sub(old_stake - new_stake)?
    };

    // Update the total of all stakes above min_bond.
    // Some variables and consts for readability
    let previously_above_min_bond = old_stake >= cfg.min_bond;
    let now_above_min_bond = new_stake >= cfg.min_bond;
    // Case distinction:
    match (previously_above_min_bond, now_above_min_bond) {
        (false, false) => {} // rewards power does not change, so do nothing
        (false, true) => {
            // stake was previously not counted, but should be now, so add new_stake to total
            total.powered_stake += new_stake;
        }
        (true, false) => {
            // stake was counted previously, but should not be now, so remove old_stake from total
            total.powered_stake -= old_stake;
        }
        (true, true) => {
            // stake was counted previously, but is different now, so add / remove difference to / from total
            if new_stake >= old_stake {
                total.powered_stake += new_stake - old_stake;
            } else {
                total.powered_stake -= old_stake - new_stake;
            }
        }
    }

    // save updated total
    TOTAL_PER_PERIOD.save(storage, &totals)?;

    Ok(())
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // info.sender is the address of the cw20 contract (that re-sent this message).
    // wrapper.sender is the address of the user that requested the cw20 contract to send this.
    // This cannot be fully trusted (the cw20 contract can fake it), so only use it for actions
    // in the address's favor (like paying/bonding tokens, not withdrawls)

    let msg: ReceiveMsg = from_slice(&wrapper.msg)?;
    let api = deps.api;
    match msg {
        ReceiveMsg::Delegate {
            unbonding_period,
            delegate_as,
        } => {
            if UNBOND_ALL.load(deps.storage)? {
                return Err(ContractError::CannotDelegateIfUnbondAll {});
            }
            execute_bond(
                deps,
                env,
                info.sender,
                wrapper.amount,
                unbonding_period,
                api.addr_validate(&delegate_as.unwrap_or(wrapper.sender))?,
            )
        }
        ReceiveMsg::MassDelegate {
            unbonding_period,
            delegate_to,
        } => {
            if UNBOND_ALL.load(deps.storage)? {
                return Err(ContractError::CannotDelegateIfUnbondAll {});
            }
            execute_mass_bond(
                deps,
                env,
                info.sender,
                wrapper.amount,
                unbonding_period,
                delegate_to,
            )
        }
        ReceiveMsg::Fund { funding_info } => {
            if UNBOND_ALL.load(deps.storage)? {
                return Err(ContractError::CannotDistributeIfUnbondAll {
                    what: "funds".into(),
                });
            }
            if funding_info.start_time < env.block.time.seconds() {
                return Err(ContractError::PastStartingTime {});
            }
            let validated_asset = AssetInfo::Token(info.sender.to_string()).validate(deps.api)?;
            update_reward_config(deps.storage, validated_asset, wrapper.amount, funding_info)?;
            Ok(Response::default())
        }
    }
}

pub fn execute_unbond(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    unbonding_period: u64,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    // If unbond all flag has been set to true, no unbonding period is required: !true as u64 == 0
    let unbond_all = UNBOND_ALL.load(deps.storage)?;

    remove_stake_without_total(
        deps.branch(),
        &env,
        &cfg,
        &info.sender,
        unbonding_period,
        amount,
    )?;

    // update total
    TOTAL_STAKED.update::<_, StdError>(deps.storage, |token_info| {
        Ok(TokenInfo {
            staked: token_info.staked.saturating_sub(amount),
            // If unbond all flag set to true the unbonding period is 0.
            unbonding: token_info.unbonding + Uint128::new(!unbond_all as u128) * amount,
        })
    })?;

    let resp = Response::new()
        .add_attribute("action", "unbond")
        .add_attribute("amount", amount)
        .add_attribute("sender", info.sender.clone());

    // If unbond all flag set to true we don't need to create a claim and send directly. Sending
    // directly instead of send a Claim submessage resolves in 2 messages instead of 3.
    if unbond_all {
        let msg = create_undelegate_msg(info.sender, amount, cfg.cw20_contract)?;
        Ok(resp.add_submessage(msg))
    } else {
        // provide them a claim
        CLAIMS.create_claim(
            deps.storage,
            &info.sender,
            amount,
            // If unbond all flag set to true the claim has no delay.
            Expiration::AtTime(env.block.time.plus_seconds(unbonding_period)),
        )?;
        Ok(resp)
    }
}

pub fn execute_quick_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stakers: Vec<String>,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // this can only be called if unbonder is set
    if cfg.unbonder != Some(info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let staker_addresses = validate_addresses(deps.api, &stakers)?;

    let mut response = Response::<Empty>::new()
        .add_attribute("action", "quick_unbond")
        .add_attribute("stakers", stakers.join(","));

    // Keep track of unbonded amounts per period.
    // This is used to update the total per period and the total staked amount in one go at the end
    // to avoid unnecessary stores for each staker.
    let mut unbonded_by_period = HashMap::with_capacity(cfg.unbonding_periods.len());
    for period in &cfg.unbonding_periods {
        unbonded_by_period.insert(period, Uint128::zero());
    }
    // Also keep track of the total amount of claims removed.
    let mut claimed_total = Uint128::zero();

    let mut distributions: Vec<_> = DISTRIBUTION
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    for staker in staker_addresses {
        // calculate rewards power before updating the stake
        let old_rewards = calc_rewards_powers(deps.storage, &cfg, &staker, distributions.iter())?;

        // the amount the staker unbonds in this call
        let mut staker_unbonds = Uint128::zero();

        let stakes = STAKE
            .prefix(&staker)
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?;
        for (unbonding_period, mut bonding_info) in stakes {
            let old_stake = bonding_info.total_stake();
            // increase the unbonding counter
            *unbonded_by_period.get_mut(&unbonding_period).unwrap() += old_stake;
            staker_unbonds += old_stake;
            // unlock all locked tokens and release all of them
            bonding_info.force_unlock_all()?;
            bonding_info.release_stake(&env, old_stake)?;
            STAKE.save(deps.storage, (&staker, unbonding_period), &bonding_info)?;
        }

        // update the adjustment data for all distributions
        for ((asset_info, distribution), old_reward_power) in
            distributions.iter_mut().zip(old_rewards.into_iter())
        {
            if old_reward_power.is_zero() {
                continue;
            }
            // new power is always zero, since we unbonded all stake
            update_rewards(
                deps.storage,
                asset_info,
                &staker,
                distribution,
                old_reward_power,
                Uint128::zero(),
            )?;
        }

        let open_claims: Uint128 = CLAIMS
            .query_claims(deps.as_ref(), &staker)?
            .claims
            .into_iter()
            .map(|c| c.amount)
            .sum();
        // in order to delete the claims, we need to create a new Map with the same key,
        // because the `Claims` API does not provide a way to delete unmature claims.
        const CLAIMS_MAP: Map<&Addr, Vec<Claim>> = Map::new("claims");
        CLAIMS_MAP.save(deps.storage, &staker, &vec![])?;
        claimed_total += open_claims;

        let amount = staker_unbonds + open_claims;
        if !amount.is_zero() {
            let undelegate_msg = WasmMsg::Execute {
                contract_addr: cfg.cw20_contract.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: staker.to_string(),
                    amount,
                })?,
                funds: vec![],
            };
            response = response.add_message(undelegate_msg);
        }
    }

    // only save updated distributions and totals at the end to save gas
    for (asset_info, distribution) in distributions.into_iter() {
        DISTRIBUTION.save(deps.storage, &asset_info, &distribution)?;
    }
    let unbonded_total = unbonded_by_period.values().sum::<Uint128>();
    for (unbonding_period, unbonded) in unbonded_by_period {
        update_total_stake(
            deps.storage,
            &cfg,
            *unbonding_period,
            unbonded,
            Uint128::zero(),
        )?;
    }
    TOTAL_STAKED.update::<_, StdError>(deps.storage, |token_info| {
        Ok(TokenInfo {
            staked: token_info.staked - unbonded_total,
            unbonding: token_info.unbonding - claimed_total,
        })
    })?;

    Ok(response)
}

pub fn execute_unbond_all(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    // Only unbonder can execute unbond all and set state variable to true.
    ensure_eq!(
        cfg.unbonder,
        Some(info.sender),
        ContractError::Unauthorized {}
    );

    UNBOND_ALL.update::<_, ContractError>(deps.storage, |unbond_all| {
        if !unbond_all {
            Ok(true)
        } else {
            Err(ContractError::FlagAlreadySet {})
        }
    })?;

    Ok(Response::default().add_attribute("action", "unbond all"))
}

pub fn execute_stop_unbond_all(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    if cfg.unbonder != Some(info.sender.clone()) && !ADMIN.is_admin(deps.as_ref(), &info.sender)? {
        return Err(ContractError::Unauthorized {});
    }

    UNBOND_ALL.update::<_, ContractError>(deps.storage, |unbond_all| {
        if unbond_all {
            Ok(false)
        } else {
            Err(ContractError::FlagAlreadySet {})
        }
    })?;

    Ok(Response::default().add_attribute("action", "stop unbond all"))
}

/// Calculates rewards power of the user for all given distributions (for all unbonding periods).
/// They are returned in the same order as the distributions.
fn calc_rewards_powers<'a>(
    storage: &dyn Storage,
    cfg: &Config,
    staker: &Addr,
    distributions: impl Iterator<Item = &'a (AssetInfoValidated, Distribution)>,
) -> StdResult<Vec<Uint128>> {
    // go through distributions and calculate old reward power for all of them
    let old_rewards = distributions
        .map(|(_, distribution)| {
            let old_reward_power = distribution.calc_rewards_power(storage, cfg, staker)?;
            Ok(old_reward_power)
        })
        .collect::<StdResult<Vec<_>>>()?;

    Ok(old_rewards)
}

fn update_rewards(
    storage: &mut dyn Storage,
    asset_info: &AssetInfoValidated,
    sender: &Addr,
    distribution: &mut Distribution,
    old_reward_power: Uint128,
    new_reward_power: Uint128,
) -> StdResult<()> {
    // short-circuit if no change
    if old_reward_power == new_reward_power {
        return Ok(());
    }

    // update their share of the distribution
    let ppw = distribution.shares_per_point.u128();
    let diff = new_reward_power.u128() as i128 - old_reward_power.u128() as i128;
    apply_points_correction(storage, sender, asset_info, ppw, diff)?;

    Ok(())
}

/// Removes the stake from the given unbonding period and staker,
/// updating `DISTRIBUTION`, `TOTAL_PER_PERIOD` and `STAKE`, but *not* `TOTAL_STAKED`.
fn remove_stake_without_total(
    deps: DepsMut,
    env: &Env,
    cfg: &Config,
    staker: &Addr,
    unbonding_period: UnbondingPeriod,
    amount: Uint128,
) -> Result<(), ContractError> {
    if cfg
        .unbonding_periods
        .binary_search(&unbonding_period)
        .is_err()
    {
        return Err(ContractError::NoUnbondingPeriodFound(unbonding_period));
    }

    let distributions: Vec<_> = DISTRIBUTION
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    // calculate rewards power before updating the stake
    let old_rewards = calc_rewards_powers(deps.storage, cfg, staker, distributions.iter())?;

    // reduce the sender's stake - aborting if insufficient
    let mut old_stake = Uint128::zero();
    let new_stake = STAKE
        .update(
            deps.storage,
            (staker, unbonding_period),
            |bonding_info| -> StdResult<_> {
                let mut bonding_info = bonding_info.unwrap_or_default();
                old_stake = bonding_info.total_stake();
                bonding_info.release_stake(env, amount)?;
                Ok(bonding_info)
            },
        )?
        .total_stake();

    update_total_stake(deps.storage, cfg, unbonding_period, old_stake, new_stake)?;

    // update the adjustment data for all distributions
    for ((asset_info, mut distribution), old_reward_power) in
        distributions.into_iter().zip(old_rewards.into_iter())
    {
        let new_reward_power = distribution.calc_rewards_power(deps.storage, cfg, staker)?;
        update_rewards(
            deps.storage,
            &asset_info,
            staker,
            &mut distribution,
            old_reward_power,
            new_reward_power,
        )?;

        // save updated distribution
        DISTRIBUTION.save(deps.storage, &asset_info, &distribution)?;
    }
    Ok(())
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let release = CLAIMS.claim_tokens(deps.storage, &info.sender, &env.block, None)?;
    if release.is_zero() {
        return Err(ContractError::NothingToClaim {});
    }

    let config = CONFIG.load(deps.storage)?;
    let amount_str = coin_to_string(release, config.cw20_contract.as_str());
    let undelegate_msg = create_undelegate_msg(info.sender.clone(), release, config.cw20_contract)?;

    TOTAL_STAKED.update::<_, StdError>(deps.storage, |token_info| {
        Ok(TokenInfo {
            staked: token_info.staked,
            unbonding: token_info.unbonding.saturating_sub(release),
        })
    })?;

    Ok(Response::new()
        .add_submessage(undelegate_msg)
        .add_attribute("action", "claim")
        .add_attribute("tokens", amount_str)
        .add_attribute("sender", info.sender))
}

#[inline]
fn coin_to_string(amount: Uint128, address: &str) -> String {
    format!("{} {}", amount, address)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Claims { address } => {
            to_binary(&CLAIMS.query_claims(deps, &deps.api.addr_validate(&address)?)?)
        }
        QueryMsg::Staked {
            address,
            unbonding_period,
        } => to_binary(&query_staked(deps, &env, address, unbonding_period)?),
        QueryMsg::AnnualizedRewards {} => to_binary(&query_annualized_rewards(deps, env)?),
        QueryMsg::BondingInfo {} => to_binary(&query_bonding_info(deps)?),
        QueryMsg::AllStaked { address } => to_binary(&query_all_staked(deps, env, address)?),
        QueryMsg::TotalStaked {} => to_binary(&query_total_staked(deps)?),
        QueryMsg::TotalUnbonding {} => to_binary(&query_total_unbonding(deps)?),
        QueryMsg::Admin {} => to_binary(&ADMIN.query_admin(deps)?),
        QueryMsg::TotalRewardsPower {} => to_binary(&query_total_rewards(deps)?),
        QueryMsg::RewardsPower { address } => to_binary(&query_rewards(deps, address)?),
        QueryMsg::WithdrawableRewards { owner } => {
            to_binary(&query_withdrawable_rewards(deps, owner)?)
        }
        QueryMsg::DistributedRewards {} => to_binary(&query_distributed_rewards(deps)?),
        QueryMsg::UndistributedRewards {} => to_binary(&query_undistributed_rewards(deps, env)?),
        QueryMsg::Delegated { owner } => to_binary(&query_delegated(deps, owner)?),
        QueryMsg::DistributionData {} => to_binary(&query_distribution_data(deps)?),
        QueryMsg::WithdrawAdjustmentData { addr, asset } => {
            to_binary(&query_withdraw_adjustment_data(deps, addr, asset)?)
        }
        QueryMsg::UnbondAll {} => to_binary(&query_unbond_all(deps)?),
    }
}

// this is all the info we need below
struct DistStats {
    asset: AssetInfoValidated,
    /// The total rewards power in the distribution
    total_rewards: Uint128,
    reward_multipliers: Vec<(UnbondingPeriod, Decimal)>,
    /// The amount of tokens that will (probably) be distributed by this distribution within the next year
    annualized_payout: Decimal,
}

fn query_annualized_rewards(deps: Deps, env: Env) -> StdResult<AnnualizedRewardsResponse> {
    let config = CONFIG.load(deps.storage)?;
    let now = env.block.time.seconds();

    // reward info for each distribution flow... do all the heavy calcs per distribution once.
    // we can then just read this for each unbonding period
    let distributions = DISTRIBUTION
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| {
            let (asset, d) = r?;
            let total_rewards = d.total_rewards_power(deps.storage, &config);
            let reward_multipliers = d.reward_multipliers;

            let reward_curve = REWARD_CURVE.may_load(deps.storage, &asset)?;
            let annualized_payout = calculate_annualized_payout(reward_curve, now);

            Ok(DistStats {
                asset,
                total_rewards,
                reward_multipliers,
                annualized_payout,
            })
        })
        .collect::<StdResult<Vec<_>>>()?;

    let mut aprs = Vec::with_capacity(config.unbonding_periods.len());

    for &unbonding_period in &config.unbonding_periods {
        let mut rewards = Vec::with_capacity(distributions.len());
        for stats in &distributions {
            if stats.total_rewards.is_zero() {
                rewards.push(AnnualizedReward {
                    info: stats.asset.clone(),
                    amount: None,
                });
                continue;
            }

            // we want basically, typical reward payout times the multiplier of this unbonding period
            // multiplier * annualized payout / total points
            let multiplier: Decimal = stats
                .reward_multipliers
                .iter()
                .find(|(ub, _)| ub == &unbonding_period)
                .unwrap()
                .1;
            // normalize by tokens_per_power
            let annual_rewards = (multiplier * stats.annualized_payout)
                / (stats.total_rewards * config.tokens_per_power);

            rewards.push(AnnualizedReward {
                info: stats.asset.clone(),
                amount: Some(annual_rewards),
            });
        }
        aprs.push((unbonding_period, rewards));
    }
    Ok(AnnualizedRewardsResponse { rewards: aprs })
}

fn calculate_annualized_payout(reward_curve: Option<Curve>, now: u64) -> Decimal {
    match reward_curve {
        Some(c) => {
            // look at the last timestamp in the rewards curve and extrapolate
            match c.end() {
                Some(last_timestamp) => {
                    if last_timestamp <= now {
                        return Decimal::zero();
                    }
                    let time_diff = last_timestamp - now;
                    if time_diff >= SECONDS_PER_YEAR {
                        // if the last timestamp is more than a year in the future,
                        // we can just calculate the rewards for the whole year directly

                        // formula: `(locked_now - locked_end)`
                        Decimal::from_atomics(c.value(now) - c.value(now + SECONDS_PER_YEAR), 0)
                            .expect("too many rewards")
                    } else {
                        // if the last timestamp is less than a year in the future,
                        // we want to extrapolate the rewards for the whole year

                        // formula: `(locked_now - locked_end) / time_diff * SECONDS_PER_YEAR`
                        // `locked_now - locked_end` are the tokens freed up over the `time_diff`.
                        // Dividing by that diff, gives us the rate of tokens per second,
                        // which is then extrapolated to a whole year.
                        // Because of the constraints put on `c` when setting it,
                        // we know that `locked_end` is always 0, so we don't need to subtract it.
                        Decimal::from_ratio(
                            c.value(now) * Uint128::from(SECONDS_PER_YEAR),
                            time_diff,
                        )
                    }
                }
                None => {
                    // this case should only happen if the reward curve is freshly initialized
                    // (i.e. no rewards have been scheduled yet)
                    Decimal::zero()
                }
            }
        }
        None => Decimal::zero(),
    }
}

fn query_rewards(deps: Deps, addr: String) -> StdResult<RewardsPowerResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    let rewards = DISTRIBUTION
        .range(deps.storage, None, None, Order::Ascending)
        .map(|dist| {
            let (asset_info, distribution) = dist?;
            let cfg = CONFIG.load(deps.storage)?;

            distribution
                .calc_rewards_power(deps.storage, &cfg, &addr)
                .map(|power| (asset_info, power))
        })
        .filter(|dist| matches!(dist, Ok((_, power)) if !power.is_zero()))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(RewardsPowerResponse { rewards })
}

fn query_total_rewards(deps: Deps) -> StdResult<RewardsPowerResponse> {
    Ok(RewardsPowerResponse {
        rewards: DISTRIBUTION
            .range(deps.storage, None, None, Order::Ascending)
            .map(|distr| {
                let (asset_info, distribution) = distr?;

                let cfg = CONFIG.load(deps.storage)?;
                Ok((
                    asset_info,
                    distribution.total_rewards_power(deps.storage, &cfg),
                ))
            })
            .collect::<StdResult<Vec<_>>>()?,
    })
}

fn query_bonding_info(deps: Deps) -> StdResult<BondingInfoResponse> {
    let total_stakes = TOTAL_PER_PERIOD.load(deps.storage)?;

    let bonding = total_stakes
        .into_iter()
        .map(|(unbonding_period, total_staked)| -> StdResult<_> {
            Ok(BondingPeriodInfo {
                unbonding_period,
                total_staked: total_staked.staked,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BondingInfoResponse { bonding })
}

pub fn query_staked(
    deps: Deps,
    env: &Env,
    addr: String,
    unbonding_period: u64,
) -> StdResult<StakedResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    // sanity check if such unbonding period exists
    let totals = TOTAL_PER_PERIOD.load(deps.storage)?;
    totals
        .binary_search_by_key(&unbonding_period, |&(entry, _)| entry)
        .map_err(|_| {
            StdError::generic_err(format!("No unbonding period found: {}", unbonding_period))
        })?;

    let stake = STAKE
        .may_load(deps.storage, (&addr, unbonding_period))?
        .unwrap_or_default();
    let cw20_contract = CONFIG.load(deps.storage)?.cw20_contract.to_string();
    Ok(StakedResponse {
        stake: stake.total_stake(),
        total_locked: stake.total_locked(env),
        unbonding_period,
        cw20_contract,
    })
}

pub fn query_all_staked(deps: Deps, env: Env, addr: String) -> StdResult<AllStakedResponse> {
    let addr = deps.api.addr_validate(&addr)?;
    let config = CONFIG.load(deps.storage)?;
    let cw20_contract = config.cw20_contract.to_string();

    let stakes = config
        .unbonding_periods
        .into_iter()
        .filter_map(|up| match STAKE.may_load(deps.storage, (&addr, up)) {
            Ok(Some(stake)) => Some(Ok(StakedResponse {
                stake: stake.total_stake(),
                total_locked: stake.total_locked(&env),
                unbonding_period: up,
                cw20_contract: cw20_contract.clone(),
            })),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        })
        .collect::<StdResult<Vec<StakedResponse>>>()?;

    Ok(AllStakedResponse { stakes })
}

pub fn query_total_staked(deps: Deps) -> StdResult<TotalStakedResponse> {
    Ok(TotalStakedResponse {
        total_staked: TOTAL_STAKED.load(deps.storage).unwrap_or_default().staked,
    })
}

pub fn query_total_unbonding(deps: Deps) -> StdResult<TotalUnbondingResponse> {
    Ok(TotalUnbondingResponse {
        total_unbonding: TOTAL_STAKED
            .load(deps.storage)
            .unwrap_or_default()
            .unbonding,
    })
}

pub fn query_unbond_all(deps: Deps) -> StdResult<UnbondAllResponse> {
    Ok(UnbondAllResponse {
        unbond_all: UNBOND_ALL.load(deps.storage)?,
    })
}

/// Manages the contract migration.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // add unbonder to config
    let mut config = CONFIG.load(deps.storage)?;
    config.unbonder = addr_opt_validate(deps.api, &msg.unbonder)?;
    config.converter = msg
        .converter
        .map(|c| {
            StdResult::Ok(ConverterConfig {
                contract: deps.api.addr_validate(&c.contract)?,
                pair_to: deps.api.addr_validate(&c.pair_to)?,
            })
        })
        .transpose()?;
    CONFIG.save(deps.storage, &config)?;

    // set unbond all flag
    UNBOND_ALL.save(deps.storage, &msg.unbond_all)?;

    Ok(Response::new())
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_slice, Coin, CosmosMsg, Decimal, WasmMsg};
    use cw_controllers::Claim;
    use cw_utils::Duration;
    use wyndex::asset::{native_asset_info, token_asset_info};

    use crate::error::ContractError;
    use crate::msg::{DistributionDataResponse, WithdrawAdjustmentDataResponse};
    use crate::state::{Distribution, WithdrawAdjustment};

    use super::*;

    const INIT_ADMIN: &str = "admin";
    const USER1: &str = "user1";
    const USER2: &str = "user2";
    const USER3: &str = "user3";
    const TOKENS_PER_POWER: Uint128 = Uint128::new(1_000);
    const MIN_BOND: Uint128 = Uint128::new(5_000);
    const UNBONDING_BLOCKS: u64 = 100;
    const UNBONDING_PERIOD: u64 = UNBONDING_BLOCKS / 5;
    const UNBONDING_PERIOD_2: u64 = 2 * UNBONDING_PERIOD;
    const CW20_ADDRESS: &str = "wasm1234567890";
    const DENOM: &str = "juno";

    #[test]
    fn check_crate_name() {
        assert_eq!(CONTRACT_NAME, "crates.io:wyndex_stake");
    }

    fn default_instantiate(deps: DepsMut, env: Env) {
        cw20_instantiate(
            deps,
            env,
            TOKENS_PER_POWER,
            MIN_BOND,
            vec![UNBONDING_PERIOD],
        )
    }

    fn cw20_instantiate(
        deps: DepsMut,
        env: Env,
        tokens_per_power: Uint128,
        min_bond: Uint128,
        stake_config: Vec<UnbondingPeriod>,
    ) {
        let msg = InstantiateMsg {
            cw20_contract: CW20_ADDRESS.to_owned(),
            tokens_per_power,
            min_bond,
            unbonding_periods: stake_config,
            admin: Some(INIT_ADMIN.into()),
            max_distributions: 6,
            unbonder: None,
            converter: None,
        };
        let info = mock_info("creator", &[]);
        instantiate(deps, env, info, msg).unwrap();
    }

    fn bond_cw20_with_period(
        mut deps: DepsMut,
        user1: u128,
        user2: u128,
        user3: u128,
        unbonding_period: u64,
        time_delta: u64,
    ) {
        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(time_delta);

        for (addr, stake) in &[(USER1, user1), (USER2, user2), (USER3, user3)] {
            if *stake != 0 {
                let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
                    sender: addr.to_string(),
                    amount: Uint128::new(*stake),
                    msg: to_binary(&ReceiveMsg::Delegate {
                        unbonding_period,
                        delegate_as: None,
                    })
                    .unwrap(),
                });
                let info = mock_info(CW20_ADDRESS, &[]);
                execute(deps.branch(), env.clone(), info, msg).unwrap();
            }
        }
    }

    fn bond_cw20(deps: DepsMut, user1: u128, user2: u128, user3: u128, time_delta: u64) {
        bond_cw20_with_period(deps, user1, user2, user3, UNBONDING_PERIOD, time_delta);
    }

    fn rebond_with_period(
        mut deps: DepsMut,
        user1: u128,
        user2: u128,
        user3: u128,
        bond_from: u64,
        bond_to: u64,
        time_delta: u64,
    ) {
        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(time_delta);

        for (addr, stake) in &[(USER1, user1), (USER2, user2), (USER3, user3)] {
            if *stake != 0 {
                let msg = ExecuteMsg::Rebond {
                    bond_from,
                    bond_to,
                    tokens: Uint128::new(*stake),
                };
                let info = mock_info(addr, &[]);
                execute(deps.branch(), env.clone(), info, msg).unwrap();
            }
        }
    }

    fn unbond_with_period(
        mut deps: DepsMut,
        user1: u128,
        user2: u128,
        user3: u128,
        time_delta: u64,
        unbonding_period: u64,
    ) {
        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(time_delta);

        for (addr, stake) in &[(USER1, user1), (USER2, user2), (USER3, user3)] {
            if *stake != 0 {
                let msg = ExecuteMsg::Unbond {
                    tokens: Uint128::new(*stake),
                    unbonding_period,
                };
                let info = mock_info(addr, &[]);
                execute(deps.branch(), env.clone(), info, msg).unwrap();
            }
        }
    }

    fn unbond(deps: DepsMut, user1: u128, user2: u128, user3: u128, time_delta: u64) {
        unbond_with_period(deps, user1, user2, user3, time_delta, UNBONDING_PERIOD);
    }

    fn native(denom: &str) -> AssetInfoValidated {
        AssetInfoValidated::Native(denom.to_string())
    }

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        default_instantiate(deps.as_mut(), env);

        // it worked, let's query the state
        let res = ADMIN.query_admin(deps.as_ref()).unwrap();
        assert_eq!(Some(INIT_ADMIN.into()), res.admin);

        // setup distribution flow
        execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            native_asset_info(DENOM),
            vec![(UNBONDING_PERIOD, Decimal::percent(1))],
        )
        .unwrap();

        // make sure distribution logic is set up properly
        let raw = query(deps.as_ref(), mock_env(), QueryMsg::DistributionData {}).unwrap();
        let res: DistributionDataResponse = from_slice(&raw).unwrap();
        assert_eq!(
            res.distributions,
            vec![(
                AssetInfoValidated::Native(DENOM.to_string()),
                Distribution {
                    shares_per_point: Uint128::zero(),
                    shares_leftover: 0,
                    distributed_total: Uint128::zero(),
                    withdrawable_total: Uint128::zero(),
                    manager: Addr::unchecked(INIT_ADMIN),
                    reward_multipliers: vec![(UNBONDING_PERIOD, Decimal::percent(1))],
                }
            )]
        );

        let raw = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::WithdrawAdjustmentData {
                addr: USER1.to_owned(),
                asset: native_asset_info(DENOM),
            },
        )
        .unwrap();
        let res: WithdrawAdjustmentDataResponse = from_slice(&raw).unwrap();
        assert_eq!(
            res,
            WithdrawAdjustment {
                shares_correction: 0,
                withdrawn_rewards: Uint128::zero(),
            }
        );
    }

    fn assert_stake_in_period(
        deps: Deps,
        env: &Env,
        user1_stake: u128,
        user2_stake: u128,
        user3_stake: u128,
        unbonding_period: u64,
    ) {
        let stake1 = query_staked(deps, env, USER1.into(), unbonding_period).unwrap();
        assert_eq!(stake1.stake.u128(), user1_stake);

        let stake2 = query_staked(deps, env, USER2.into(), unbonding_period).unwrap();
        assert_eq!(stake2.stake.u128(), user2_stake);

        let stake3 = query_staked(deps, env, USER3.into(), unbonding_period).unwrap();
        assert_eq!(stake3.stake.u128(), user3_stake);
    }

    // this tests the member queries
    fn assert_stake(
        deps: Deps,
        env: &Env,
        user1_stake: u128,
        user2_stake: u128,
        user3_stake: u128,
    ) {
        assert_stake_in_period(
            deps,
            env,
            user1_stake,
            user2_stake,
            user3_stake,
            UNBONDING_PERIOD,
        );
    }

    fn assert_cw20_undelegate(res: cosmwasm_std::Response, recipient: &str, amount: u128) {
        match &res.messages[0].msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                msg,
                funds,
            }) => {
                assert_eq!(contract_addr.as_str(), CW20_ADDRESS);
                assert_eq!(funds.len(), 0);
                let parsed: Cw20ExecuteMsg = from_slice(msg).unwrap();
                assert_eq!(
                    parsed,
                    Cw20ExecuteMsg::Transfer {
                        recipient: recipient.into(),
                        amount: Uint128::new(amount)
                    }
                );
            }
            _ => panic!("Must initiate undelegate!"),
        }
    }

    fn assert_native_rewards(
        response: Vec<(AssetInfoValidated, Uint128)>,
        expected: &[(&str, u128)],
        msg: &str,
    ) {
        assert_eq!(
            expected
                .iter()
                .map(|(denom, power)| (native(denom), Uint128::new(*power)))
                .collect::<Vec<_>>(),
            response,
            "{}",
            msg
        );
    }

    #[test]
    fn cw20_token_bond() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        cw20_instantiate(
            deps.as_mut(),
            env.clone(),
            TOKENS_PER_POWER,
            MIN_BOND,
            vec![UNBONDING_PERIOD],
        );

        // ensure it rounds down, and respects cut-off
        bond_cw20(deps.as_mut(), 12_000, 7_500, 4_000, 1);

        // Assert updated powers
        assert_stake(deps.as_ref(), &env, 12_000, 7_500, 4_000);
    }

    #[test]
    fn cw20_token_claim() {
        let unbonding_period: u64 = 20;

        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let unbonding = Duration::Time(unbonding_period);
        cw20_instantiate(
            deps.as_mut(),
            env.clone(),
            TOKENS_PER_POWER,
            MIN_BOND,
            vec![unbonding_period],
        );

        // bond some tokens
        bond_cw20(deps.as_mut(), 20_000, 13_500, 500, 5);

        // unbond part
        unbond(deps.as_mut(), 7_900, 4_600, 0, unbonding_period);

        // Assert updated powers
        assert_stake(deps.as_ref(), &env, 12_100, 8_900, 500);

        // with proper claims
        env.block.time = env.block.time.plus_seconds(unbonding_period);
        let expires = unbonding.after(&env.block);
        assert_eq!(
            get_claims(deps.as_ref(), &Addr::unchecked(USER1)),
            vec![Claim::new(7_900, expires)]
        );

        // wait til they expire and get payout
        env.block.time = env.block.time.plus_seconds(unbonding_period);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            mock_info(USER1, &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);

        assert_stake(deps.as_ref(), &env, 12_100, 8_900, 500);
        assert_cw20_undelegate(res, USER1, 7_900)
    }

    fn get_claims(deps: Deps, addr: &Addr) -> Vec<Claim> {
        CLAIMS.query_claims(deps, addr).unwrap().claims
    }

    #[test]
    fn unbond_claim_workflow() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        default_instantiate(deps.as_mut(), env.clone());

        // create some data
        bond_cw20(deps.as_mut(), 12_000, 7_500, 4_000, 5);
        unbond(deps.as_mut(), 4_500, 2_600, 0, 10);
        env.block.time = env.block.time.plus_seconds(10);

        // check the claims for each user
        let expires = Duration::Time(UNBONDING_PERIOD).after(&env.block);
        assert_eq!(
            get_claims(deps.as_ref(), &Addr::unchecked(USER1)),
            vec![Claim::new(4_500, expires)]
        );
        assert_eq!(
            get_claims(deps.as_ref(), &Addr::unchecked(USER2)),
            vec![Claim::new(2_600, expires)]
        );
        assert_eq!(get_claims(deps.as_ref(), &Addr::unchecked(USER3)), vec![]);

        // do another unbond later on
        let mut env2 = mock_env();
        env2.block.time = env2.block.time.plus_seconds(22);
        unbond(deps.as_mut(), 0, 1_345, 1_500, 22);

        // with updated claims
        let expires2 = Duration::Time(UNBONDING_PERIOD).after(&env2.block);
        assert_eq!(
            get_claims(deps.as_ref(), &Addr::unchecked(USER1)),
            vec![Claim::new(4_500, expires)]
        );
        assert_eq!(
            get_claims(deps.as_ref(), &Addr::unchecked(USER2)),
            vec![Claim::new(2_600, expires), Claim::new(1_345, expires2)]
        );
        assert_eq!(
            get_claims(deps.as_ref(), &Addr::unchecked(USER3)),
            vec![Claim::new(1_500, expires2)]
        );

        // nothing can be withdrawn yet
        let err = execute(
            deps.as_mut(),
            env2,
            mock_info(USER1, &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::NothingToClaim {});

        // now mature first section, withdraw that
        let mut env3 = mock_env();
        env3.block.time = env3.block.time.plus_seconds(UNBONDING_PERIOD + 10);
        // first one can now release
        let res = execute(
            deps.as_mut(),
            env3.clone(),
            mock_info(USER1, &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap();
        assert_cw20_undelegate(res, USER1, 4_500);

        // second releases partially
        let res = execute(
            deps.as_mut(),
            env3.clone(),
            mock_info(USER2, &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap();
        assert_cw20_undelegate(res, USER2, 2_600);

        // but the third one cannot release
        let err = execute(
            deps.as_mut(),
            env3,
            mock_info(USER3, &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap_err();
        assert_eq!(err, ContractError::NothingToClaim {});

        // claims updated properly
        assert_eq!(get_claims(deps.as_ref(), &Addr::unchecked(USER1)), vec![]);
        assert_eq!(
            get_claims(deps.as_ref(), &Addr::unchecked(USER2)),
            vec![Claim::new(1_345, expires2)]
        );
        assert_eq!(
            get_claims(deps.as_ref(), &Addr::unchecked(USER3)),
            vec![Claim::new(1_500, expires2)]
        );

        // add another few claims for 2
        unbond(deps.as_mut(), 0, 600, 0, 6 + UNBONDING_PERIOD);
        unbond(deps.as_mut(), 0, 1_005, 0, 10 + UNBONDING_PERIOD);

        // ensure second can claim all tokens at once
        let mut env4 = mock_env();
        env4.block.time = env4.block.time.plus_seconds(UNBONDING_PERIOD * 2 + 12);
        let res = execute(
            deps.as_mut(),
            env4,
            mock_info(USER2, &[]),
            ExecuteMsg::Claim {},
        )
        .unwrap();
        assert_cw20_undelegate(res, USER2, 2_950); // 1_345 + 600 + 1_005
        assert_eq!(get_claims(deps.as_ref(), &Addr::unchecked(USER2)), vec![]);
    }

    fn rewards(deps: Deps, user: &str) -> Vec<(AssetInfoValidated, Uint128)> {
        query_rewards(deps, user.to_string()).unwrap().rewards
    }

    #[test]
    fn rewards_saved() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        cw20_instantiate(
            deps.as_mut(),
            env,
            TOKENS_PER_POWER,
            MIN_BOND,
            vec![UNBONDING_PERIOD],
        );

        // create distribution flow to be able to receive rewards
        execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            native_asset_info(DENOM),
            vec![(UNBONDING_PERIOD, Decimal::percent(1))],
        )
        .unwrap();

        // assert original rewards
        assert_eq!(rewards(deps.as_ref(), USER1), vec![]);
        assert_eq!(rewards(deps.as_ref(), USER2), vec![]);
        assert_eq!(rewards(deps.as_ref(), USER3), vec![]);

        // ensure it rounds down, and respects cut-off
        bond_cw20(deps.as_mut(), 1_200_000, 770_000, 4_000_000, 1);

        // assert updated rewards
        assert_native_rewards(
            rewards(deps.as_ref(), USER1),
            &[(DENOM, 12)],
            "1_200_000 * 1% / 1_000 = 12",
        );
        assert_native_rewards(
            rewards(deps.as_ref(), USER2),
            &[(DENOM, 7)],
            "770_000 * 1% / 1_000 = 7",
        );
        assert_native_rewards(
            rewards(deps.as_ref(), USER3),
            &[(DENOM, 40)],
            "4_000_000 * 1% / 1_000 = 40",
        );

        // unbond some tokens
        unbond(deps.as_mut(), 100_000, 99_600, 3_600_000, UNBONDING_PERIOD);

        assert_native_rewards(
            rewards(deps.as_ref(), USER1),
            &[(DENOM, 11)],
            "1_100_000 * 1% / 1_000 = 11",
        );
        assert_native_rewards(
            rewards(deps.as_ref(), USER2),
            &[(DENOM, 6)],
            "600_955 * 1% / 1_000 = 6",
        );
        // USER3 has 400_000 left, this is above min_bound. But the rewards (4_000) would have been less
        assert_native_rewards(
            rewards(deps.as_ref(), USER3),
            &[(DENOM, 4)],
            "min_bound applied to stake (400_000), before reward multiplier (4_000)",
        );
    }

    #[test]
    fn rewards_rebonding() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        cw20_instantiate(
            deps.as_mut(),
            env.clone(),
            TOKENS_PER_POWER,
            Uint128::new(1000),
            vec![UNBONDING_PERIOD, UNBONDING_PERIOD_2],
        );

        // create distribution flow to be able to receive rewards
        execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            native_asset_info(DENOM),
            vec![
                (UNBONDING_PERIOD, Decimal::percent(1)),
                (UNBONDING_PERIOD_2, Decimal::percent(10)),
            ],
        )
        .unwrap();

        // assert original rewards
        assert_eq!(rewards(deps.as_ref(), USER1), vec![]);
        assert_eq!(rewards(deps.as_ref(), USER2), vec![]);
        assert_eq!(rewards(deps.as_ref(), USER3), vec![]);

        // bond some tokens for first period
        bond_cw20(deps.as_mut(), 1_000_000, 180_000, 10_000, 1);

        // assert updated rewards
        assert_native_rewards(
            rewards(deps.as_ref(), USER1),
            &[(DENOM, 10)],
            "1_000_000 * 1% / 1_000 = 10",
        );
        assert_native_rewards(
            rewards(deps.as_ref(), USER2),
            &[(DENOM, 1)],
            "180_000 * 1% / 1_000 = 1",
        );
        assert_native_rewards(
            rewards(deps.as_ref(), USER3),
            &[],
            "10_000 * 1% = 100 < min_bond",
        );

        // bond some more tokens for second period
        bond_cw20_with_period(
            deps.as_mut(),
            1_000_000,
            100_000,
            9_000,
            UNBONDING_PERIOD_2,
            2,
        );

        // assert updated rewards
        assert_native_rewards(
            rewards(deps.as_ref(), USER1),
            &[(DENOM, 110)],
            "10 + 1_000_000 * 10% / 1_000 = 110",
        );
        assert_native_rewards(
            rewards(deps.as_ref(), USER2),
            &[(DENOM, 11)],
            "1 + 100_000 * 10% / 1_000 = 11",
        );
        assert_native_rewards(
            rewards(deps.as_ref(), USER3),
            &[],
            "0 + 9_000 * 10% = 900 < min_bond",
        );

        // rebond tokens
        rebond_with_period(
            deps.as_mut(),
            100_000,
            180_000,
            10_000,
            UNBONDING_PERIOD,
            UNBONDING_PERIOD_2,
            3,
        );

        // assert stake
        assert_stake(deps.as_ref(), &env, 900_000, 0, 0);
        assert_stake_in_period(
            deps.as_ref(),
            &env,
            1_100_000,
            280_000,
            19_000,
            UNBONDING_PERIOD_2,
        );
        // assert updated rewards
        assert_native_rewards(
            rewards(deps.as_ref(), USER1),
            &[(DENOM, 119)],
            "900_000 * 1% / 1_000 + 1_100_000 * 10% / 1_000 = 9 + 110 = 119",
        );
        assert_native_rewards(
            rewards(deps.as_ref(), USER2),
            &[(DENOM, 28)],
            "0 + 280_000 * 10% / 1_000 = 28",
        );
        assert_native_rewards(
            rewards(deps.as_ref(), USER3),
            &[(DENOM, 1)],
            "0 + 19_000 * 10% / 1_000 = 1",
        );
    }

    #[test]
    fn ensure_bonding_edge_cases() {
        // use min_bond 0, tokens_per_power 500
        let mut deps = mock_dependencies();
        let env = mock_env();
        cw20_instantiate(
            deps.as_mut(),
            env,
            Uint128::new(100),
            Uint128::zero(),
            vec![UNBONDING_PERIOD],
        );

        // setting 50 tokens, gives us None power
        bond_cw20(deps.as_mut(), 50, 1, 102, 1);

        // reducing to 0 token makes us None even with min_bond 0
        unbond(deps.as_mut(), 49, 1, 102, 2);
    }

    #[test]
    fn test_query_bonding_info() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut(), mock_env());

        let bonding_info_response = query_bonding_info(deps.as_ref()).unwrap();
        assert_eq!(
            bonding_info_response,
            BondingInfoResponse {
                bonding: vec!(BondingPeriodInfo {
                    unbonding_period: 20,
                    total_staked: Uint128::zero(),
                })
            }
        );
    }

    #[test]
    fn max_distribution_limit() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut(), mock_env());

        // create distribution flows up to the maximum
        const DENOMS: [&str; 6] = ["a", "b", "c", "d", "e", "f"];
        for denom in &DENOMS {
            execute_create_distribution_flow(
                deps.as_mut(),
                mock_info(INIT_ADMIN, &[]),
                INIT_ADMIN.to_string(),
                native_asset_info(denom),
                vec![(UNBONDING_PERIOD, Decimal::one())],
            )
            .unwrap();
        }
        // next one should fail
        let err = execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            native_asset_info(DENOM),
            vec![(UNBONDING_PERIOD, Decimal::one())],
        )
        .unwrap_err();
        assert_eq!(err, ContractError::TooManyDistributions(6));
    }

    #[test]
    fn distribution_already_exists() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut(), mock_env());

        // create distribution flow
        execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            native_asset_info(DENOM),
            vec![(UNBONDING_PERIOD, Decimal::one())],
        )
        .unwrap();

        // next one should fail
        let err = execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            native_asset_info(DENOM),
            vec![(UNBONDING_PERIOD, Decimal::one())],
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::DistributionAlreadyExists(AssetInfoValidated::Native(
                "juno".to_string()
            ))
        );
    }

    #[test]
    fn distribute_unsupported_token_fails() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut(), mock_env());

        // create distribution flow
        execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            native_asset_info(DENOM),
            vec![(UNBONDING_PERIOD, Decimal::one())],
        )
        .unwrap();

        // call distribute, but send unsupported funds
        let unsupported_funds = Coin {
            denom: "unsupported".to_string(),
            amount: Uint128::new(100),
        };
        let err = execute_distribute_rewards(
            deps.as_mut(),
            mock_env(),
            mock_info(INIT_ADMIN, &[unsupported_funds.clone()]),
            None,
        )
        .unwrap_err();

        assert_eq!(err, ContractError::NoDistributionFlow(unsupported_funds));
    }

    #[test]
    fn cannot_distribute_staking_token() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut(), mock_env());

        // try to create distribution flow for staking token
        let err = execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            token_asset_info(CW20_ADDRESS),
            vec![(UNBONDING_PERIOD, Decimal::one())],
        )
        .unwrap_err();

        assert_eq!(err, ContractError::InvalidAsset {});
    }

    #[test]
    fn cannot_distribute_staking_token_without_enough_per_block() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut(), mock_env());

        // try to create distribution flow for staking token
        let _res = execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            native_asset_info(DENOM),
            vec![(UNBONDING_PERIOD, Decimal::one())],
        )
        .unwrap();
        let err = execute_fund_distribution(
            mock_env(),
            deps.as_mut(),
            mock_info(
                INIT_ADMIN,
                &[Coin {
                    denom: DENOM.to_string(),
                    amount: Uint128::zero(),
                }],
            ),
            FundingInfo {
                start_time: mock_env().block.time.seconds(),
                distribution_duration: mock_env().block.time.seconds() + 10u64,
                amount: Uint128::new(1),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::InvalidRewards {});
    }

    #[test]
    fn distribution_flow_wrong_unbonding_period_fails() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut(), mock_env());

        // try to create distribution flow with wrong unbonding period
        let err = execute_create_distribution_flow(
            deps.as_mut(),
            mock_info(INIT_ADMIN, &[]),
            INIT_ADMIN.to_string(),
            native_asset_info(DENOM),
            vec![(UNBONDING_PERIOD + 1, Decimal::one())],
        )
        .unwrap_err();
        assert_eq!(err, ContractError::InvalidRewards {});
    }

    #[test]
    fn delegate_as_someone_else() {
        let mut deps = mock_dependencies();
        default_instantiate(deps.as_mut(), mock_env());

        execute_receive(
            deps.as_mut(),
            mock_env(),
            mock_info(CW20_ADDRESS, &[]),
            Cw20ReceiveMsg {
                sender: "delegator".to_string(),
                amount: 100u128.into(),
                msg: to_binary(&ReceiveMsg::Delegate {
                    unbonding_period: UNBONDING_PERIOD,
                    delegate_as: Some("owner_of_stake".to_string()),
                })
                .unwrap(),
            },
        )
        .unwrap();

        // owner_of_stake should have the stake
        let stake = query_staked(
            deps.as_ref(),
            &mock_env(),
            "owner_of_stake".to_string(),
            UNBONDING_PERIOD,
        )
        .unwrap()
        .stake
        .u128();
        assert_eq!(stake, 100u128);
    }
}
