use crate::msg::StakingAction;
use abstract_staking_adapter_traits::CwStakingError;
use cosmwasm_std::{DepsMut, Env, MessageInfo, SubMsg};

use abstract_staking_adapter_traits::CwStakingCommand;

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
            StakingAction::Stake { stake } => provider.stake(deps.as_ref(), stake)?,
            StakingAction::Unstake { unstake } => provider.unstake(deps.as_ref(), unstake)?,
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
fn staking_assets_from_action(action: &StakingAction) -> impl Iterator<Item = AssetEntry> {
    match action {
        StakingAction::Stake { stake } => stake.iter().map(|req| req.asset.name.clone()).into(),
        StakingAction::Unstake { unstake } => {
            unstake.iter().map(|req| req.asset.name.clone()).into()
        }
        StakingAction::ClaimRewards {
            assets: staking_token,
        } => staking_token.into(),
        StakingAction::Claim {
            assets: staking_token,
        } => staking_token.into(),
    }
}
