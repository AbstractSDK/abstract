#![warn(missing_docs)]
use cosmwasm_std::{Addr, Uint128};
use cw_asset::AssetInfo;
use cw_utils::{Duration, Expiration};

/// Possible staking targets to support staking on cosmwasm contract or cosmos Lockup module
#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum StakingTarget {
    Contract(Addr),
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

#[cosmwasm_schema::cw_serde]
pub struct StakingInfoResponse {
    pub staking_target: StakingTarget,
    pub staking_token: AssetInfo,
    pub unbonding_periods: Option<Vec<Duration>>,
    pub max_claims: Option<u32>,
}

#[cosmwasm_schema::cw_serde]
pub struct StakeResponse {
    pub amount: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct RewardTokensResponse {
    pub tokens: Vec<AssetInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct UnbondingResponse {
    pub claims: Vec<Claim>,
}

#[cosmwasm_schema::cw_serde]
pub struct Claim {
    pub amount: Uint128,
    pub claimable_at: Expiration,
}
