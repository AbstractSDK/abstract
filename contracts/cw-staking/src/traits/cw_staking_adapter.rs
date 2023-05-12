use crate::contract::CwStakingResult;
use crate::error::StakingError;
use crate::msg::{RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse};
use crate::traits::identify::Identify;
use abstract_sdk::core::objects::{AssetEntry, ContractEntry};
use abstract_sdk::feature_objects::AnsHost;
use abstract_sdk::AbstractSdkResult;
use cosmwasm_std::{Addr, CosmosMsg, Deps, Env, QuerierWrapper, Uint128};
use cw_utils::Duration;

/// Trait that defines the adapter interface for staking providers
pub trait StakingCommand: Identify {
    /// Construct a staking contract entry from the staking token and the provider
    fn staking_entry(&self, staking_token: &AssetEntry) -> ContractEntry {
        ContractEntry {
            protocol: self.name().to_string(),
            contract: format!("staking/{staking_token}"),
        }
    }

    /// Retrieve the staking contract address for the pool with the provided staking token name
    fn staking_contract_address(
        &self,
        deps: Deps,
        ans_host: &AnsHost,
        token: &AssetEntry,
    ) -> AbstractSdkResult<Addr> {
        let staking_contract = self.staking_entry(token);

        ans_host
            .query_contract(&deps.querier, &staking_contract)
            .map_err(Into::into)
    }

    /// Fetch the required data for interacting with the provider
    fn fetch_data(
        &mut self,
        deps: Deps,
        env: Env,
        ans_host: &AnsHost,
        staking_asset: AssetEntry,
    ) -> AbstractSdkResult<()>;

    /// Stake the provided asset into the staking contract
    fn stake(
        &self,
        deps: Deps,
        amount: Uint128,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, StakingError>;

    /// Stake the provided asset into the staking contract
    fn unstake(
        &self,
        deps: Deps,
        amount: Uint128,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, StakingError>;

    /// Claim rewards on the staking contract
    ///
    /// * `deps` - the dependencies
    fn claim_rewards(&self, deps: Deps) -> Result<Vec<CosmosMsg>, StakingError>;

    /// Claim matured unbonding claims on the staking contract
    fn claim(&self, deps: Deps) -> Result<Vec<CosmosMsg>, StakingError>;

    fn query_info(&self, querier: &QuerierWrapper) -> CwStakingResult<StakingInfoResponse>;

    /// Query the staked token balance of a given staker
    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        unbonding_period: Option<Duration>,
    ) -> CwStakingResult<StakeResponse>;

    /// Query unbonding information of a given staker
    fn query_unbonding(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
    ) -> CwStakingResult<UnbondingResponse>;

    /// Query for pending claimable rewards
    fn query_rewards(&self, querier: &QuerierWrapper) -> CwStakingResult<RewardTokensResponse>;
}
