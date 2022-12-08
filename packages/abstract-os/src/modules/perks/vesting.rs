use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod state {
    use super::*;
    pub const CONFIG: Item<Config> = Item::new("config");
    pub const STATE: Item<State> = Item::new("state");
    pub const ALLOCATIONS: Map<&Addr, AllocationInfo> = Map::new("vested_allocations");

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        /// Account which can create new allocations
        pub owner: Addr,
        /// Account which will receive refunds upon allocation terminations
        pub refund_recipient: Addr,
        /// Address of token
        pub token: Addr,
        /// By default, unlocking starts at launch, with a cliff of 12 months and a duration of 12 months.
        /// If not specified, all allocations use this default schedule
        pub default_unlock_schedule: Schedule,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
    pub struct State {
        /// Tokens deposited into the contract
        pub total_deposited: Uint128,
        /// Currently available Tokens
        pub remaining_tokens: Uint128,
    }

    impl Default for State {
        fn default() -> Self {
            State {
                total_deposited: Uint128::zero(),
                remaining_tokens: Uint128::zero(),
            }
        }
    }
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Account which can create new allocations
    pub owner: String,
    /// Account which will receive refunds upon allocation terminations
    pub refund_recipient: String,
    /// Address of tokens token
    pub token: String,
    /// By default, unlocking starts at init, with a cliff of 12 months and a duration of 12 months.
    /// If not specified, all allocations use this default schedule
    pub default_unlock_schedule: Schedule,
}

#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Admin function. Update addresses of owner
    TransferOwnership { new_owner: String },
    /// Admin function. Implementation of cw20 receive msg to create new allocations
    Receive(Cw20ReceiveMsg),
    /// Claim withdrawable tokens
    Withdraw {},
    /// Terminates the allocation
    Terminate { user_address: String },
}

#[cosmwasm_schema::cw_serde]
pub enum ReceiveMsg {
    /// Create new allocations
    CreateAllocations {
        allocations: Vec<(String, AllocationInfo)>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Config of this contract
    #[returns(ConfigResponse)]
    Config {},
    // State of this contract
    #[returns(StateResponse)]
    State {},
    // Parameters and current status of an allocation
    #[returns(AllocationResponse)]
    Allocation { account: String },
    // Simulate how many tokens will be released if a withdrawal is attempted
    #[returns(SimulateWithdrawResponse)]
    SimulateWithdraw {
        account: String,
        timestamp: Option<u64>,
    },
}

pub type ConfigResponse = InstantiateMsg;
pub type AllocationResponse = AllocationInfo;

#[cosmwasm_schema::cw_serde]
pub struct StateResponse {
    /// tokens Tokens deposited into the contract
    pub total_deposited: Uint128,
    /// Currently available tokens Tokens
    pub remaining_tokens: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct SimulateWithdrawResponse {
    /// Total number of tokens tokens allocated to this account
    pub total_tokens_locked: Uint128,
    /// Total number of tokens tokens that have been unlocked till now
    pub total_tokens_unlocked: Uint128,
    /// Total number of tokens tokens that have been vested till now
    pub total_tokens_vested: Uint128,
    /// Number of tokens tokens that have been withdrawn by the beneficiary
    pub withdrawn_amount: Uint128,
    /// Number of tokens tokens that can be withdrawn by the beneficiary post the provided timestamp
    pub withdrawable_amount: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct AllocationInfo {
    /// Total number of tokens tokens allocated to this account
    pub total_amount: Uint128,
    ///  Number of tokens tokens that have been withdrawn by the beneficiary
    pub withdrawn_amount: Uint128,
    /// Parameters controlling the vesting process
    pub vest_schedule: Schedule,
    /// Parameters controlling the unlocking process
    pub unlock_schedule: Option<Schedule>,
    /// Indicates if this vesting allo has been canceled
    pub canceled: bool,
}

// Parameters describing a typical vesting schedule
#[cosmwasm_schema::cw_serde]
pub struct Schedule {
    /// Timestamp of when vesting is to be started
    pub start_time: u64,
    /// Number of seconds starting UST during which no token will be vested/unlocked
    pub cliff: u64,
    /// Number of seconds taken by tokens to be fully vested
    pub duration: u64,
}

impl Schedule {
    pub fn zero() -> Schedule {
        Schedule {
            start_time: 0u64,
            cliff: 0u64,
            duration: 0u64,
        }
    }
}
