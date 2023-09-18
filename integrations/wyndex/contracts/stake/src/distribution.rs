use std::collections::HashSet;

use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Storage, Uint128};
use wyndex::asset::{AssetInfo, AssetInfoExt, AssetInfoValidated};

use crate::error::ContractError;
use crate::msg::{
    DelegatedResponse, DistributedRewardsResponse, DistributionDataResponse,
    UndistributedRewardsResponse, WithdrawAdjustmentDataResponse, WithdrawableRewardsResponse,
};
use crate::state::{
    Config, Distribution, WithdrawAdjustment, CONFIG, DELEGATED, DISTRIBUTION, REWARD_CURVE,
    SHARES_SHIFT, UNBOND_ALL, WITHDRAW_ADJUSTMENT,
};

pub fn execute_distribute_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Option<String>,
) -> Result<Response, ContractError> {
    if UNBOND_ALL.load(deps.storage)? {
        return Err(ContractError::CannotDistributeIfUnbondAll {
            what: "rewards".into(),
        });
    }

    let sender = sender
        .map(|sender| deps.api.addr_validate(&sender))
        .transpose()?
        .unwrap_or(info.sender);

    let distributions = DISTRIBUTION
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    // do not accept unsupported funds
    // we can only check the ones that were sent with the message (so only native assets)
    let supported_assets = distributions
        .iter()
        .filter_map(|(a, _)| a.native_denom())
        .collect::<HashSet<_>>();
    if let Some(unsupported_coin) = info
        .funds
        .iter()
        .find(|c| !supported_assets.contains(c.denom.as_str()))
    {
        return Err(ContractError::NoDistributionFlow(unsupported_coin.clone()));
    }

    let mut resp = Response::new()
        .add_attribute("action", "distribute_rewards")
        .add_attribute("sender", sender.as_str());

    let cfg = CONFIG.load(deps.storage)?;
    for (asset_info, mut distribution) in distributions {
        let total_rewards = distribution.total_rewards_power(deps.storage, &cfg);
        // There are no shares in play - noone to distribute to
        if total_rewards.is_zero() {
            continue;
        }

        let withdrawable: u128 = distribution.withdrawable_total.into();

        // Query current reward balance
        let balance =
            undistributed_rewards(deps.as_ref(), &asset_info, env.contract.address.clone())?.u128();

        let curve = REWARD_CURVE.load(deps.storage, &asset_info)?;

        // Calculate how much we have received since the last time Distributed was called,
        // including only the reward config amount that is eligible for distribution.
        // This is the amount we will distribute to all members.
        let amount = balance - withdrawable - curve.value(env.block.time.seconds()).u128();

        if amount == 0 {
            continue;
        }

        let leftover: u128 = distribution.shares_leftover.into();
        let points = (amount << SHARES_SHIFT) + leftover;
        let points_per_share = points / total_rewards.u128();
        distribution.shares_leftover = (points % total_rewards.u128()) as u64;

        // Everything goes back to 128-bits/16-bytes
        // Full amount is added here to total withdrawable, as it should not be considered on its own
        // on future distributions - even if because of calculation offsets it is not fully
        // distributed, the error is handled by leftover.
        distribution.shares_per_point += Uint128::new(points_per_share);
        distribution.distributed_total += Uint128::new(amount);
        distribution.withdrawable_total += Uint128::new(amount);

        DISTRIBUTION.save(deps.storage, &asset_info, &distribution)?;

        resp = resp.add_attribute(format!("amount_{}", asset_info), amount.to_string());
    }

    Ok(resp)
}

/// Query current reward balance of the given asset.
/// Make sure not to call this for the staking token
fn undistributed_rewards(
    deps: Deps,
    asset_info: &AssetInfoValidated,
    contract_address: impl Into<String>,
) -> StdResult<Uint128> {
    asset_info.query_balance(&deps.querier, contract_address)
}

pub fn execute_withdraw_rewards(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    receiver: Option<String>,
) -> Result<Response, ContractError> {
    let owner = owner.map_or_else(
        || Ok(info.sender.clone()),
        |owner| deps.api.addr_validate(&owner),
    )?;
    let receiver = receiver
        .map(|receiver| deps.api.addr_validate(&receiver))
        .transpose()?
        .unwrap_or_else(|| info.sender.clone());

    let mut resp = Response::new()
        .add_attribute("action", "withdraw_rewards")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("owner", owner.as_str())
        .add_attribute("receiver", receiver.as_str());

    let distributions = DISTRIBUTION
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    let delegated = DELEGATED
        .may_load(deps.storage, &owner)?
        .unwrap_or_else(|| owner.clone());
    if ![&owner, &delegated].contains(&&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let cfg = CONFIG.load(deps.storage)?;
    for (asset_info, mut distribution) in distributions {
        // get adjustment data
        let mut adjustment = WITHDRAW_ADJUSTMENT
            .may_load(deps.storage, (&owner, &asset_info))?
            .unwrap_or_default();

        let reward = withdrawable_rewards(deps.as_ref(), &cfg, &owner, &distribution, &adjustment)?;

        if reward.is_zero() {
            // Just do nothing
            continue;
        }
        adjustment.withdrawn_rewards += reward;
        WITHDRAW_ADJUSTMENT.save(deps.storage, (&owner, &asset_info), &adjustment)?;
        distribution.withdrawable_total -= reward;
        DISTRIBUTION.save(deps.storage, &asset_info, &distribution)?;
        // send rewards to receiver
        let msg = asset_info.with_balance(reward).into_msg(receiver.clone())?;

        resp = resp
            .add_message(msg)
            .add_attribute(format!("reward_{}", asset_info), reward);
    }

    Ok(resp)
}

pub fn execute_delegate_withdrawal(
    deps: DepsMut,
    info: MessageInfo,
    delegated: String,
) -> Result<Response, ContractError> {
    let delegated = deps.api.addr_validate(&delegated)?;

    DELEGATED.save(deps.storage, &info.sender, &delegated)?;
    let resp = Response::new()
        .add_attribute("action", "delegate_withdrawal")
        .add_attribute("sender", info.sender.as_str())
        .add_attribute("delegated", &delegated);

    Ok(resp)
}

pub fn query_withdrawable_rewards(
    deps: Deps,
    owner: String,
) -> StdResult<WithdrawableRewardsResponse> {
    // Not checking address, as if it is invalid it is guaranteed not to appear in maps, so
    // `withdrawable_rewards` would return error itself.
    let owner = Addr::unchecked(owner);

    let cfg = CONFIG.load(deps.storage)?;
    let distributions =
        DISTRIBUTION.range(deps.storage, None, None, cosmwasm_std::Order::Ascending);

    let rewards = distributions
        .map(|distr| -> StdResult<_> {
            let (asset_info, distribution) = distr?;
            let adjustment = WITHDRAW_ADJUSTMENT
                .may_load(deps.storage, (&owner, &asset_info))?
                .unwrap_or_default();
            let rewards = withdrawable_rewards(deps, &cfg, &owner, &distribution, &adjustment)?;

            Ok(asset_info.with_balance(rewards))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(WithdrawableRewardsResponse { rewards })
}

pub fn query_undistributed_rewards(
    deps: Deps,
    env: Env,
) -> StdResult<UndistributedRewardsResponse> {
    let distributions =
        DISTRIBUTION.range(deps.storage, None, None, cosmwasm_std::Order::Ascending);

    let rewards = distributions
        .map(|distribution| -> StdResult<_> {
            let (asset_info, distribution) = distribution?;
            let balance = undistributed_rewards(deps, &asset_info, env.contract.address.clone())?;

            Ok(asset_info.with_balance(balance - distribution.withdrawable_total))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(UndistributedRewardsResponse { rewards })
}

pub fn query_distributed_rewards(deps: Deps) -> StdResult<DistributedRewardsResponse> {
    let distributions = DISTRIBUTION
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(DistributedRewardsResponse {
        distributed: distributions
            .iter()
            .map(|(asset_info, dist)| asset_info.with_balance(dist.distributed_total))
            .collect(),
        withdrawable: distributions
            .iter()
            .map(|(asset_info, dist)| asset_info.with_balance(dist.withdrawable_total))
            .collect(),
    })
}

pub fn query_delegated(deps: Deps, owner: String) -> StdResult<DelegatedResponse> {
    let owner = deps.api.addr_validate(&owner)?;

    let delegated = DELEGATED.may_load(deps.storage, &owner)?.unwrap_or(owner);

    Ok(DelegatedResponse { delegated })
}

pub fn query_distribution_data(deps: Deps) -> StdResult<DistributionDataResponse> {
    Ok(DistributionDataResponse {
        distributions: DISTRIBUTION
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?,
    })
}

pub fn query_withdraw_adjustment_data(
    deps: Deps,
    owner: String,
    asset: AssetInfo,
) -> StdResult<WithdrawAdjustmentDataResponse> {
    let addr = deps.api.addr_validate(&owner)?;
    let asset = asset.validate(deps.api)?;
    let adjust = WITHDRAW_ADJUSTMENT
        .may_load(deps.storage, (&addr, &asset))?
        .unwrap_or(WithdrawAdjustmentDataResponse {
            shares_correction: 0,
            withdrawn_rewards: Uint128::zero(),
        });
    Ok(adjust)
}

/// Applies points correction for given address.
/// `shares_per_point` is current value from `SHARES_PER_POINT` - not loaded in function, to
/// avoid multiple queries on bulk updates.
/// `diff` is the points change
pub fn apply_points_correction(
    storage: &mut dyn Storage,
    addr: &Addr,
    asset_info: &AssetInfoValidated,
    shares_per_point: u128,
    diff: i128,
) -> StdResult<()> {
    WITHDRAW_ADJUSTMENT.update(storage, (addr, asset_info), |old| -> StdResult<_> {
        let mut old = old.unwrap_or_default();
        let shares_correction: i128 = old.shares_correction;
        old.shares_correction = shares_correction - shares_per_point as i128 * diff;
        Ok(old)
    })?;
    Ok(())
}

/// This is customized for the use case of the contract
/// Since asset is clear from the distribution, we just return the number
pub fn withdrawable_rewards(
    deps: Deps,
    cfg: &Config,
    owner: &Addr,
    distribution: &Distribution,
    adjustment: &WithdrawAdjustment,
) -> StdResult<Uint128> {
    let ppw = distribution.shares_per_point.u128();
    let points = distribution
        .calc_rewards_power(deps.storage, cfg, owner)?
        .u128();

    let correction = adjustment.shares_correction;
    let points = (ppw * points) as i128;
    let points = points + correction;
    let amount = points as u128 >> SHARES_SHIFT;
    let amount = amount - adjustment.withdrawn_rewards.u128();

    Ok(amount.into())
}
