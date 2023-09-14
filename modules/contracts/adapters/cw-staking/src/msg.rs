#![warn(missing_docs)]
//! # Staking Adapter
//!
//! `abstract::cw-staking`
use crate::contract::CwStakingAdapter;
use abstract_core::objects::{AnsAsset, AssetEntry};
use cosmwasm_schema::QueryResponses;
use cw_utils::Duration;
// Re-export response types
pub use abstract_staking_adapter_traits::types::*;

/// Name of the staking provider, used by the ANS.
pub type ProviderName = String;

/// The callback id for staking over ibc
pub const IBC_STAKING_PROVIDER_ID: u32 = 22335;

abstract_adapter::adapter_msg_types!(CwStakingAdapter, StakingExecuteMsg, StakingQueryMsg);

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
        /// The ANS-resolvable asset information of the asset to stake.
        asset: AnsAsset,
        /// The unbonding period for the specified stake.
        unbonding_period: Option<Duration>,
    },
    /// Unstake/unbond a given token
    Unstake {
        /// The ANS-resolvable asset information of the asset to unstake.
        asset: AnsAsset,
        /// The unbonding period for the specified stake.
        unbonding_period: Option<Duration>,
    },
    /// Claim rewards for a given token
    ClaimRewards {
        /// Staking asset to claim rewards for.
        asset: AssetEntry,
    },

    /// Claim matured unbonding tokens
    Claim {
        /// Unbonded staking asset to claim.
        asset: AssetEntry,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
/// Query messages for the staking adapter
pub enum StakingQueryMsg {
    /// Get the staking info for a given provider
    #[returns(StakingInfoResponse)]
    Info {
        /// Name of the provider
        provider: ProviderName,
        /// The staking token to query
        staking_token: AssetEntry,
    },
    /// Get the staked amount for a given provider, staking token, staker address and unbonding period
    #[returns(StakeResponse)]
    Staked {
        /// Name of the provider
        provider: ProviderName,
        /// The staking token to query
        staking_token: AssetEntry,
        /// The address of the staker (contract or user)
        staker_address: String,
        /// The unbonding period for the specified staked position.
        unbonding_period: Option<Duration>,
    },
    /// Get the unbonding entries for a given provider, staking token and staker address
    #[returns(UnbondingResponse)]
    Unbonding {
        /// Name of the provider
        provider: ProviderName,
        /// The staking token to query
        staking_token: AssetEntry,
        /// The address of the staker (contract or user)
        staker_address: String,
    },
    /// Get the reward tokens for a given provider and staking token
    #[returns(RewardTokensResponse)]
    RewardTokens {
        /// Name of the provider
        provider: ProviderName,
        /// The staking token to query
        staking_token: AssetEntry,
    },
}
