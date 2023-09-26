use abstract_staking_standard::msg::StakingAction;
use abstract_staking_standard::CwStakingError;
use cosmwasm_std::{DepsMut, Env, MessageInfo, SubMsg};

use abstract_staking_standard::CwStakingCommand;

use abstract_sdk::{
    core::objects::AssetEntry,
    features::{AbstractNameService, AbstractRegistryAccess},
    Execution,
};

impl<T> CwStakingAdapter for T where T: AbstractNameService + AbstractRegistryAccess + Execution {}

/// Trait for dispatching *local* staking actions to the appropriate provider
/// Resolves the required data for that provider
/// Identifies an Adapter as a Staking Adapter
pub trait CwStakingAdapter: AbstractNameService + AbstractRegistryAccess + Execution {
    /// resolve the provided staking action on a local provider
    fn resolve_staking_action(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        action: StakingAction,
        mut provider: Box<dyn CwStakingCommand>,
    ) -> Result<SubMsg, CwStakingError> {
        let staking_asset = staking_assets_from_action(&action);

        provider.fetch_data(
            deps.as_ref(),
            env,
            Some(info),
            &self.ans_host(deps.as_ref())?,
            &self.abstract_registry(deps.as_ref())?,
            staking_asset,
        )?;

        let msgs = match action {
            StakingAction::Stake {
                assets,
                unbonding_period,
            } => provider.stake(deps.as_ref(), assets, unbonding_period)?,
            StakingAction::Unstake {
                assets,
                unbonding_period,
            } => provider.unstake(deps.as_ref(), assets, unbonding_period)?,
            StakingAction::ClaimRewards { assets: _ } => provider.claim_rewards(deps.as_ref())?,
            StakingAction::Claim { assets: _ } => provider.claim(deps.as_ref())?,
        };

        self.executor(deps.as_ref())
            .execute(msgs.into_iter().map(Into::into).collect())
            .map(SubMsg::new)
            .map_err(Into::into)
    }
}

#[inline(always)]
fn staking_assets_from_action(action: &StakingAction) -> Vec<AssetEntry> {
    match action {
        StakingAction::Stake {
            assets: staking_tokens,
            ..
        } => staking_tokens.iter().map(|req| req.name.clone()).collect(),
        StakingAction::Unstake {
            assets: staking_tokens,
            ..
        } => staking_tokens.iter().map(|req| req.name.clone()).collect(),
        StakingAction::ClaimRewards {
            assets: staking_tokens,
        } => staking_tokens.clone(),
        StakingAction::Claim {
            assets: staking_token,
        } => staking_token.clone(),
    }
}
