//! # Staking Adapter
//!
//! `4t2::cw-staking`

use abstract_core::adapter;
use abstract_core::objects::{AnsAsset, AssetEntry};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty, Uint128};
use cw20::Expiration;
use cw_asset::AssetInfo;
use cw_utils::Duration;

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
    Stake {
        asset: AnsAsset,
        unbonding_period: Option<Duration>,
    },

    /// Unstake/unbond a given token
    Unstake {
        asset: AnsAsset,
        unbonding_period: Option<Duration>,
    },

    /// Claim rewards for a given token
    ClaimRewards { asset: AssetEntry },

    /// Claim matured unbonding tokens
    Claim { asset: AssetEntry },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "cw-orch", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "cw-orch", impl_into(QueryMsg))]
pub enum StakingQueryMsg {
    #[returns(StakingInfoResponse)]
    Info {
        provider: ProviderName,
        staking_token: AssetEntry,
    },
    #[returns(StakeResponse)]
    Staked {
        provider: ProviderName,
        staking_token: AssetEntry,
        staker_address: String,
        unbonding_period: Option<Duration>,
    },
    #[returns(UnbondingResponse)]
    Unbonding {
        provider: ProviderName,
        staking_token: AssetEntry,
        staker_address: String,
    },
    #[returns(RewardTokensResponse)]
    RewardTokens {
        provider: ProviderName,
        staking_token: AssetEntry,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct StakingInfoResponse {
    pub staking_contract_address: Addr,
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
