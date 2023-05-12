use crate::error::StakingError;
use crate::msg::CwStakingAction;
use crate::traits::command::StakingCommand;
use abstract_sdk::core::objects::AssetEntry;
use abstract_sdk::features::AbstractNameService;
use abstract_sdk::Execution;
use cosmwasm_std::{DepsMut, Env, SubMsg};

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
        action: CwStakingAction,
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
            CwStakingAction::Stake {
                staking_token,
                unbonding_period,
            } => provider.stake(deps.as_ref(), staking_token.amount, unbonding_period)?,
            CwStakingAction::Unstake {
                staking_token,
                unbonding_period,
            } => provider.unstake(deps.as_ref(), staking_token.amount, unbonding_period)?,
            CwStakingAction::ClaimRewards { staking_token: _ } => {
                provider.claim_rewards(deps.as_ref())?
            }
            CwStakingAction::Claim { staking_token: _ } => provider.claim(deps.as_ref())?,
        };

        self.executor(deps.as_ref())
            .execute(msgs)
            .map(SubMsg::new)
            .map_err(Into::into)
    }
}

#[inline(always)]
fn staking_asset_from_action(action: &CwStakingAction) -> AssetEntry {
    match action {
        CwStakingAction::Stake { staking_token, .. } => staking_token.name.clone(),
        CwStakingAction::Unstake { staking_token, .. } => staking_token.name.clone(),
        CwStakingAction::ClaimRewards { staking_token } => staking_token.clone(),
        CwStakingAction::Claim { staking_token } => staking_token.clone(),
    }
}
