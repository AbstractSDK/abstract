use crate::msg::{RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse};
use crate::{CwStakingError, Identify};
use abstract_sdk::core::objects::{AssetEntry, ContractEntry};
use abstract_sdk::feature_objects::{AnsHost, VersionControlContract};
use abstract_sdk::AbstractSdkResult;
use cosmwasm_std::{Addr, CosmosMsg, Deps, Env, QuerierWrapper, Uint128};
use cw_utils::Duration;
use std::error::Error;

/// Trait that defines the staking commands for providers
pub trait CwStakingCommand<E: Error = CwStakingError>: Identify {
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
        info: Option<cosmwasm_std::MessageInfo>,
        ans_host: &AnsHost,
        abstract_registry: &VersionControlContract,
        staking_asset: AssetEntry,
    ) -> AbstractSdkResult<()>;

    /// Stake the provided asset into the staking contract
    fn stake(
        &self,
        deps: Deps,
        amount: Uint128,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, E>;

    /// Stake the provided asset into the staking contract
    fn unstake(
        &self,
        deps: Deps,
        amount: Uint128,
        unbonding_period: Option<Duration>,
    ) -> Result<Vec<CosmosMsg>, E>;

    /// Claim rewards on the staking contract
    fn claim_rewards(&self, deps: Deps) -> Result<Vec<CosmosMsg>, E>;

    /// Claim matured unbonding claims on the staking contract
    fn claim(&self, deps: Deps) -> Result<Vec<CosmosMsg>, E>;

    /// Query information of the given for the given staking provider see [StakingInfoResponse]
    fn query_info(&self, querier: &QuerierWrapper) -> Result<StakingInfoResponse, E>;

    /// Query the staked token balance of a given staker
    /// This will not return  the amount of tokens that are currently unbonding.
    /// For unbonding positions, please see [Self::query_unbonding]
    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        unbonding_period: Option<Duration>,
    ) -> Result<StakeResponse, E>;

    /// Query information on unbonding positions for a given staker.
    fn query_unbonding(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
    ) -> Result<UnbondingResponse, E>;

    /// Query the information of the reward tokens
    fn query_rewards(&self, querier: &QuerierWrapper) -> Result<RewardTokensResponse, E>;
}
