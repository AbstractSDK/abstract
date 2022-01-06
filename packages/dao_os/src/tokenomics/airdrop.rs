use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub whale_token_address: String,
    pub merkle_roots: Option<Vec<String>>,
    pub from_timestamp: Option<u64>,
    pub to_timestamp: u64,
    pub total_airdrop_size: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Admin function to update the configuration parameters
    UpdateConfig {
        owner: Option<String>,
        merkle_roots: Option<Vec<String>>,
        from_timestamp: Option<u64>,
        to_timestamp: Option<u64>,
    },
    /// Allows Terra users to claim their WHALE Airdrop
    Claim {
        claim_amount: Uint128,
        merkle_proof: Vec<String>,
        root_index: u32,
    },
    /// Admin function to facilitate transfer of the unclaimed WHALE Tokens
    TransferUnclaimedTokens { recepient: String, amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    UserInfo { address: String },
    HasUserClaimed { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub whale_token_address: String,
    pub merkle_roots: Vec<String>,
    pub from_timestamp: u64,
    pub to_timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_airdrop_size: Uint128,
    pub unclaimed_tokens: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfoResponse {
    pub airdrop_amount: Uint128,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimResponse {
    pub is_claimed: bool,
}
