use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

//----------------------------------------------------------------------------------------
// Struct's :: Contract State
//----------------------------------------------------------------------------------------

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const STAKER_INFO: Map<&Addr, StakerInfo> = Map::new("staker");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Account who can update config
    pub owner: Addr,
    /// WHALE Token address
    pub whale_token: Addr,
    ///  WHALE-UST LP token address - accepted by the contract via Cw20ReceiveMsg function
    pub staking_token: Addr,
    pub staking_token_decimals: u8,
    /// Distribution Schedule
    pub distribution_schedule: (u64, u64, Uint128),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// Timestamp at which the global_reward_index was last updated
    pub last_distributed: u64,
    /// Total number of WHALE-UST LP tokens staked with the contract
    pub total_bond_amount: Uint128,
    /// Used to calculate WHALE rewards accured over time elapsed. Ratio =  Total distributed WHALE tokens / total bond amount
    pub global_reward_index: Decimal,
    /// Number of WHALE tokens that are yet to be distributed
    pub leftover: Uint128,
    /// Number of WHALE tokens distributed per staked LP token
    pub reward_rate_per_token: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    /// Number of WHALE-UST LP tokens staked by the user
    pub bond_amount: Uint128,
    /// Used to calculate WHALE rewards accured over time elapsed. Ratio = distributed WHALE tokens / user's bonded amount
    pub reward_index: Decimal,
    /// Pending WHALE tokens which are yet to be claimed
    pub pending_reward: Uint128,
}

impl Default for StakerInfo {
    fn default() -> Self {
        StakerInfo {
            reward_index: Decimal::one(),
            bond_amount: Uint128::zero(),
            pending_reward: Uint128::zero(),
        }
    }
}
