#![warn(missing_docs)]
//! # Staking Adapter types
use cosmwasm_std::{Addr, Uint128};
use cw_asset::AssetInfo;
use cw_utils::{Duration, Expiration};

/// Possible staking targets to support staking on cosmwasm contract or cosmos Lockup module
#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum StakingTarget {
    /// Address of the staking contract (Cosmwasm)
    Contract(Addr),
    /// Pool id of the staking contract (Osmosis)
    Id(u64),
}

impl StakingTarget {
    /// Extract contract address
    pub fn expect_contract(self) -> abstract_core::AbstractResult<Addr> {
        match self {
            StakingTarget::Contract(addr) => Ok(addr),
            _ => Err(abstract_core::AbstractError::Assert(
                "Staking target is not a contract address.".into(),
            )),
        }
    }

    /// Extract pool id
    pub fn expect_id(self) -> abstract_core::AbstractResult<u64> {
        match self {
            StakingTarget::Id(id) => Ok(id),
            _ => Err(abstract_core::AbstractError::Assert(
                "Staking target is not an pool ID.".into(),
            )),
        }
    }
}

impl From<u64> for StakingTarget {
    fn from(value: u64) -> Self {
        Self::Id(value)
    }
}

impl From<Addr> for StakingTarget {
    fn from(value: Addr) -> Self {
        Self::Contract(value)
    }
}

/// Response for the staking_info query
#[cosmwasm_schema::cw_serde]
pub struct StakingInfoResponse {
    /// Contract or pool id to stake to
    pub staking_target: StakingTarget,
    /// Staking token
    pub staking_token: AssetInfo,
    /// Different supported unbonding periods. None if no unbonding is supported.
    pub unbonding_periods: Option<Vec<Duration>>,
    /// Max number of claims. None if no limit.
    pub max_claims: Option<u32>,
}

/// Response for the staked query
#[cosmwasm_schema::cw_serde]
pub struct StakeResponse {
    /// Amount of staked tokens
    pub amount: Uint128,
}

/// Response for the rewards query
#[cosmwasm_schema::cw_serde]
pub struct RewardTokensResponse {
    /// List of reward tokens
    pub tokens: Vec<AssetInfo>,
}

/// Response for the unbonding query
#[cosmwasm_schema::cw_serde]
pub struct UnbondingResponse {
    /// List of unbonding entries
    pub claims: Vec<Claim>,
}

/// A claim for a given amount of tokens that are unbonding.
#[cosmwasm_schema::cw_serde]
pub struct Claim {
    /// Amount of tokens that are unbonding
    pub amount: Uint128,
    /// When the tokens can be claimed
    pub claimable_at: Expiration,
}
