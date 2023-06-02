use cosmwasm_std::{DepsMut, Env, SubMsg};

use crate::{error::StakingError, msg::StakingAction, traits::command::StakingCommand};

use abstract_sdk::{core::objects::AssetEntry, features::AbstractNameService, Execution};

impl<T> StakingAdapter for T where T: AbstractNameService + Execution {}

/// Trait for dispatching *local* staking actions to the appropriate provider
/// Resolves the required data for that provider
/// Identifies an Adapter as a Staking Adapter
pub trait StakingAdapter: AbstractNameService + Execution {
    /// resolve the provided staking action on a local provider
    fn resolve_staking_action(
        &self,
        deps: DepsMut,
        env: Env,
        action: StakingAction,
        mut provider: Box<dyn StakingCommand>,
    ) -> Result<SubMsg, StakingError> {
        let staking_asset = staking_asset_from_action(&action);

        provider.fetch_data(
            deps.as_ref(),
            env,
            &self.ans_host(deps.as_ref())?,
            staking_asset,
        )?;

        let msgs = match action {
            StakingAction::Stake {
                asset: staking_token,
                unbonding_period,
            } => provider.stake(deps.as_ref(), staking_token.amount, unbonding_period)?,
            StakingAction::Unstake {
                asset: staking_token,
                unbonding_period,
            } => provider.unstake(deps.as_ref(), staking_token.amount, unbonding_period)?,
            StakingAction::ClaimRewards { asset: _ } => provider.claim_rewards(deps.as_ref())?,
            StakingAction::Claim { asset: _ } => provider.claim(deps.as_ref())?,
        };

        self.executor(deps.as_ref())
            .execute(msgs.into_iter().map(Into::into).collect())
            .map(SubMsg::new)
            .map_err(Into::into)
    }
}

#[inline(always)]
fn staking_asset_from_action(action: &StakingAction) -> AssetEntry {
    match action {
        StakingAction::Stake {
            asset: staking_token,
            ..
        } => staking_token.name.clone(),
        StakingAction::Unstake {
            asset: staking_token,
            ..
        } => staking_token.name.clone(),
        StakingAction::ClaimRewards {
            asset: staking_token,
        } => staking_token.clone(),
        StakingAction::Claim {
            asset: staking_token,
        } => staking_token.clone(),
    }
}
