use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Account which can create new allocations
    pub owner: String,
    /// Account which will receive refunds upon allocation terminations
    pub refund_recepient: String,
    /// Address of  token
    pub token: String,
    /// By default, unlocking starts at White launch, with a cliff of 12 months and a duration of 12 months.
    /// If not specified, all allocations use this default schedule
    pub default_unlock_schedule: Schedule,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Admin function. Update addresses of owner
    TransferOwnership { new_owner: String },
    /// Admin function. Implementation of cw20 receive msg to create new allocations
    Receive(Cw20ReceiveMsg),
    /// Claim withdrawable
    Withdraw {},
    /// Terminates the allocation
    Terminate { user_address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    /// Create new allocations
    CreateAllocations {
        allocations: Vec<(String, AllocationInfo)>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Config of this contract
    Config {},
    // State of this contract
    State {},
    // Parameters and current status of an allocation
    Allocation {
        account: String,
    },
    // Simulate how many  will be released if a withdrawal is attempted
    SimulateWithdraw {
        account: String,
        timestamp: Option<u64>,
    },
}

pub type ConfigResponse = InstantiateMsg;
pub type AllocationResponse = AllocationInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    ///  Tokens deposited into the contract
    pub total_deposited: Uint128,
    /// Currently available  Tokens
    pub remaining_tokens: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SimulateWithdrawResponse {
    /// Total number of  tokens allocated to this account
    pub total_locked: Uint128,
    /// Total number of  tokens that have been unlocked till now
    pub total_unlocked: Uint128,
    /// Total number of  tokens that have been vested till now
    pub total_vested: Uint128,
    /// Number of  tokens that have been withdrawn by the beneficiary
    pub withdrawn_amount: Uint128,
    /// Number of  tokens that can be withdrawn by the beneficiary post the provided timestamp
    pub withdrawable_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllocationInfo {
    /// Total number of  tokens allocated to this account
    pub total_amount: Uint128,
    ///  Number of  tokens that have been withdrawn by the beneficiary
    pub withdrawn_amount: Uint128,
    /// Parameters controlling the vesting process
    pub vest_schedule: Schedule,
    /// Parameters controlling the unlocking process
    pub unlock_schedule: Option<Schedule>,
}

// Parameters describing a typical vesting schedule
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Schedule {
    /// Timestamp of when vesting is to be started
    pub start_time: u64,
    /// Number of seconds starting UST during which no token will be vested/unlocked
    pub cliff: u64,
    /// Number of seconds taken by tokens to be fully vested
    pub duration: u64,
}
