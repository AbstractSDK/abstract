#![warn(missing_docs)]
//! # Staking Adapter
//!
//! `abstract::cw-staking`
use abstract_core::{
    adapter,
    objects::{AnsAsset, AssetEntry},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Empty;
use cw_utils::Duration;

/// Name of the staking provider, used by the ANS.
pub type ProviderName = String;

/// The callback id for staking over ibc
pub const IBC_STAKING_PROVIDER_ID: &str = "IBC_STAKING_ACTION";

/// Top-level Abstract Adapter execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
pub type ExecuteMsg = adapter::ExecuteMsg<StakingExecuteMsg>;
/// Top-level Abstract Adapter instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
pub type InstantiateMsg = adapter::InstantiateMsg<Empty>;
/// Top-level Abstract Adapter query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
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

/// Possible actions to perform on the staking contract
/// All provide an asset [AnsAsset] information
#[cosmwasm_schema::cw_serde]
pub enum StakingAction {
    /// Stakes/bonds a given token
    Stake {
        /// The ANS-resolvable asset information of the assets to stake.
        assets: Vec<AnsAsset>,
        /// The unbonding period for the specified stake.
        unbonding_period: Option<Duration>,
    },
    /// Unstake/unbond a given token
    Unstake {
        /// The ANS-resolvable asset information of the assets to unstake.
        assets: Vec<AnsAsset>,
        /// The unbonding period for the specified stake.
        unbonding_period: Option<Duration>,
    },
    /// Claim rewards for a set of staked assets.
    ClaimRewards {
        /// Staked assets to claim rewards for.
        assets: Vec<AssetEntry>,
    },
    /// Claim matured unbonding tokens
    Claim {
        /// Unbonded staking assets to claim.
        assets: Vec<AssetEntry>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
#[impl_into(QueryMsg)]
/// Query messages for the staking adapter
pub enum StakingQueryMsg {
    /// Get the staking info for a given provider
    /// Returns [`StakingInfoResponse`]
    #[returns(StakingInfoResponse)]
    Info {
        /// Name of the provider
        provider: ProviderName,
        /// The staking tokens to query
        staking_tokens: Vec<AssetEntry>,
    },
    /// Get the staked amount for a given provider, staking token, staker address and unbonding period
    /// Returns [`StakeResponse`]
    #[returns(StakeResponse)]
    Staked {
        /// Name of the provider
        provider: ProviderName,
        /// The staking token to query
        stakes: Vec<AssetEntry>,
        /// The address of the staker (contract or user)
        staker_address: String,
        /// The unbonding period for the specified staked position.
        unbonding_period: Option<Duration>,
    },
    /// Get the unbonding entries for a given provider, staking token and staker address
    /// Returns [`UnbondingResponse`]
    #[returns(UnbondingResponse)]
    Unbonding {
        /// Name of the provider
        provider: ProviderName,
        /// The staking tokens to query
        staking_tokens: Vec<AssetEntry>,
        /// The address of the staker (contract or user)
        staker_address: String,
    },
    /// Get the reward tokens for a given provider and staking token
    /// Returns [`RewardTokensResponse`]
    #[returns(RewardTokensResponse)]
    RewardTokens {
        /// Name of the provider
        provider: ProviderName,
        /// The staking tokens to query
        staking_tokens: Vec<AssetEntry>,
    },
}

use cosmwasm_std::{Addr, Uint128};
use cw_asset::AssetInfo;
use cw_utils::Expiration;

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
    pub infos: Vec<StakingInfo>,
}

/// Info for a stakeable token
#[cosmwasm_schema::cw_serde]
pub struct StakingInfo {
    /// Address or pool id to stake to
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
    /// Amount of staked tokens, per token provided in query
    pub amounts: Vec<Uint128>,
}

/// Response for the rewards query
#[cosmwasm_schema::cw_serde]
pub struct RewardTokensResponse {
    /// List of reward tokens, per token provided in query
    pub tokens: Vec<Vec<AssetInfo>>,
}

/// Response for the unbonding query
#[cosmwasm_schema::cw_serde]
pub struct UnbondingResponse {
    /// List of unbonding entries, per token provided in query
    pub claims: Vec<Vec<Claim>>,
}

/// A claim for a given amount of tokens that are unbonding.
#[cosmwasm_schema::cw_serde]
pub struct Claim {
    /// Amount of tokens that are unbonding
    pub amount: Uint128,
    /// When the tokens can be claimed
    pub claimable_at: Expiration,
}
