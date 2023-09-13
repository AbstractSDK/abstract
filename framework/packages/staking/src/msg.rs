//! # Staking Adapter
//!
//! `4t2::cw-staking`

use cosmwasm_std::{Addr, Empty, Uint128};

use cw_asset::AssetInfo;

use cw_utils::{Duration, Expiration};

use abstract_core::adapter;
use abstract_core::objects::{AnsAsset, AssetEntry};

use cosmwasm_schema::QueryResponses;

pub type ProviderName = String;

/// The callback id for staking over ibc
pub const IBC_STAKING_PROVIDER_ID: u32 = 22335;

pub type ExecuteMsg = adapter::ExecuteMsg<StakingExecuteMsg>;
pub type InstantiateMsg = adapter::InstantiateMsg<Empty>;
pub type QueryMsg = adapter::QueryMsg<StakingQueryMsg>;

impl adapter::AdapterExecuteMsg for StakingExecuteMsg {}

impl adapter::AdapterQueryMsg for StakingQueryMsg {}

/// A request message that's sent to this staking adapter
#[cosmwasm_schema::cw_serde]
pub struct StakingExecuteMsg {
    /// The name of the staking provider
    pub provider: ProviderName,
    /// the action to execute, see [StakingAction]
    pub action: StakingAction,
}

#[cosmwasm_schema::cw_serde]
/// Possible actions to perform on the staking contract
/// All provide an asset [AnsAsset] information
pub enum StakingAction {
    /// Stakes/bonds a given token
    Stake { stake: Vec<StakeRequest> },

    /// Unstake/unbond a given token
    Unstake { unstake: Vec<StakeRequest> },

    /// Claim rewards for a given token
    ClaimRewards { assets: Vec<AssetEntry> },

    /// Claim matured unbonding tokens
    Claim { assets: Vec<AssetEntry> },
}

#[cosmwasm_schema::cw_serde]
pub struct StakeRequest {
    pub asset: AnsAsset,
    pub unbonding_period: Option<Duration>,
}

pub type UnstakeRequest = StakeRequest;

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
pub enum StakingQueryMsg {
    #[returns(StakingInfoResponse)]
    Info {
        provider: ProviderName,
        staking_tokens: Vec<AssetEntry>,
    },
    #[returns(StakeResponse)]
    Staked {
        provider: ProviderName,
        staker_address: String,
        stakes: Vec<StakedQuery>,
    },
    #[returns(UnbondingResponse)]
    Unbonding {
        provider: ProviderName,
        staker_address: String,
        staking_tokens: Vec<AssetEntry>,
    },
    #[returns(RewardTokensResponse)]
    RewardTokens {
        provider: ProviderName,
        staking_tokens: Vec<AssetEntry>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct StakedQuery {
    pub staking_token: AssetEntry,
    pub unbonding_period: Option<Duration>,
}

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
    pub infos: Vec<StakingInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct StakingInfo {
    pub staking_target: StakingTarget,
    pub staking_token: AssetInfo,
    pub unbonding_periods: Option<Vec<Duration>>,
    pub max_claims: Option<u32>,
}

#[cosmwasm_schema::cw_serde]
pub struct StakeResponse {
    pub amounts: Vec<Uint128>,
}

#[cosmwasm_schema::cw_serde]
pub struct RewardTokensResponse {
    pub tokens: Vec<(StakingTarget, Vec<AssetInfo>)>,
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
