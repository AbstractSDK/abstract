use abstract_sdk::feature_objects::{AnsHost, RegistryContract};
use abstract_std::objects::{AnsAsset, AssetEntry, ContractEntry};
use cosmwasm_std::{Addr, CosmosMsg, Deps, Env, QuerierWrapper};

use crate::{
    msg::{RewardTokensResponse, StakeResponse, StakingInfoResponse, UnbondingResponse},
    CwStakingError, Identify,
};

/// Trait that defines the staking commands for providers
pub trait CwStakingCommand: Identify {
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
    ) -> Result<Addr, CwStakingError> {
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
        addr_as_sender: Option<Addr>,
        ans_host: &AnsHost,
        version_control_contract: RegistryContract,
        staking_assets: Vec<AssetEntry>,
    ) -> Result<(), CwStakingError>;

    /// Stake the provided asset into the staking contract
    fn stake(
        &self,
        deps: Deps,
        stake_request: Vec<AnsAsset>,
        unbonding_period: Option<cw_utils::Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError>;

    /// Unstake the provided asset from the staking contract
    fn unstake(
        &self,
        deps: Deps,
        unstake_request: Vec<AnsAsset>,
        unbonding_period: Option<cw_utils::Duration>,
    ) -> Result<Vec<CosmosMsg>, CwStakingError>;

    /// Claim rewards on the staking contract
    fn claim_rewards(&self, deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError>;

    /// Claim matured unbonding claims on the staking contract
    fn claim(&self, deps: Deps) -> Result<Vec<CosmosMsg>, CwStakingError>;

    /// Query information of the given for the given staking provider see [StakingInfoResponse]
    fn query_info(&self, querier: &QuerierWrapper) -> Result<StakingInfoResponse, CwStakingError>;

    /// Query the staked token balance of a given staker
    /// This will not return  the amount of tokens that are currently unbonding.
    /// For unbonding positions, please see [Self::query_unbonding]
    fn query_staked(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
        stakes: Vec<AssetEntry>,
        unbonding_period: Option<cw_utils::Duration>,
    ) -> Result<StakeResponse, CwStakingError>;

    /// Query information on unbonding positions for a given staker.
    fn query_unbonding(
        &self,
        querier: &QuerierWrapper,
        staker: Addr,
    ) -> Result<UnbondingResponse, CwStakingError>;

    /// Query the information of the reward tokens
    fn query_rewards(
        &self,
        querier: &QuerierWrapper,
    ) -> Result<RewardTokensResponse, CwStakingError>;
}
