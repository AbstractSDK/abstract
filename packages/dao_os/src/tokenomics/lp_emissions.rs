use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Account who can update config
    pub owner: String,
    /// WHALE Token address
    pub whale_token: String,
    ///  WHALE-UST LP token address - accepted by the contract via Cw20ReceiveMsg function
    pub staking_token: String,
    pub staking_token_decimals: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Open a new user position or add to an existing position
    /// @dev Increase the total LP shares Bonded by equal no. of shares as sent by the user
    Receive(Cw20ReceiveMsg),
    /// @param new_owner The new owner address
    UpdateConfig { new_owner: String },
    /// Decrease the total LP shares Bonded by the user
    /// Accrued rewards are claimed along-with this function
    /// @param amount The no. of LP shares to be subtracted from the total Bonded and sent back to the user
    Unbond {
        amount: Uint128,
        withdraw_pending_reward: Option<bool>,
    },
    /// Claim pending rewards
    Claim {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// Open a new user position or add to an existing position (Cw20ReceiveMsg)
    Bond {},
    UpdateRewardSchedule {
        period_start: u64,
        period_finish: u64,
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the contract configuration
    Config {},
    /// Returns the global state of the contract
    /// @param timestamp Optional value which can be passed to calculate global_reward_index at a certain timestamp
    State { timestamp: Option<u64> },
    /// Returns the state of a user's staked position (StakerInfo)
    /// @param timestamp Optional value which can be passed to calculate reward_index, pending_reward at a certain timestamp
    StakerInfo {
        staker: String,
        timestamp: Option<u64>,
    },
    /// Helper function, returns the current timestamp
    Timestamp {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    /// Account who can update config
    pub owner: String,
    /// Contract used to query addresses related to red-bank
    pub whale_token: String,
    ///  WHALE-UST LP token address
    pub staking_token: String,
    /// Distribution Schedules
    pub distribution_schedule: (u64, u64, Uint128),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    /// Timestamp at which the global_reward_index was last updated
    pub last_distributed: u64,
    /// Total number of WHALE-UST LP tokens deposited in the contract
    pub total_bond_amount: Uint128,
    ///  total WHALE rewards / total_bond_amount ratio. Used to calculate WHALE rewards accured over time elapsed
    pub global_reward_index: Decimal,
    /// Number of WHALE tokens that are yet to be distributed
    pub leftover: Uint128,
    /// Number of WHALE tokens distributed per staked LP tokens
    pub reward_rate_per_token: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfoResponse {
    /// User address
    pub staker: String,
    /// WHALE-UST LP tokens deposited by the user
    pub bond_amount: Uint128,
    /// WHALE rewards / bond_amount ratio.  Used to calculate WHALE rewards accured over time elapsed
    pub reward_index: Decimal,
    /// Pending WHALE rewards which are yet to be claimed
    pub pending_reward: Uint128,
}
