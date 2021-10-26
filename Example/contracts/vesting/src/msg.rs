use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::common::OrderBy;
use cosmwasm_std::Uint128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub anchor_token: String,
    pub genesis_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<String>,
        anchor_token: Option<String>,
        genesis_time: Option<u64>,
    },
    RegisterVestingAccounts {
        vesting_accounts: Vec<VestingAccount>,
    },
    Claim {},
}

/// CONTRACT: end_time > start_time
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingAccount {
    pub address: String,
    pub schedules: Vec<(u64, u64, Uint128)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingInfo {
    pub schedules: Vec<(u64, u64, Uint128)>,
    pub last_claim_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    VestingAccount {
        address: String,
    },
    VestingAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub anchor_token: String,
    pub genesis_time: u64,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingAccountResponse {
    pub address: String,
    pub info: VestingInfo,
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VestingAccountsResponse {
    pub vesting_accounts: Vec<VestingAccountResponse>,
}
