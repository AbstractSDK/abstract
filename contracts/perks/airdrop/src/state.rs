use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG: Item<Config> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const USERS: Map<&Addr, UserInfo> = Map::new("users");

//----------------------------------------------------------------------------------------
// Storage types
//----------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Account who can update config
    pub owner: Addr,
    ///  WHALE token address
    pub whale_token_address: Addr,
    /// Merkle roots used to verify is a terra user is eligible for the airdrop
    pub merkle_roots: Vec<String>,
    /// Timestamp since which WHALE airdrops can be delegated to boostrap auction contract
    pub from_timestamp: u64,
    /// Timestamp to which WHALE airdrops can be claimed
    pub to_timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct State {
    /// Total WHALE issuance used as airdrop incentives
    pub total_airdrop_size: Uint128,
    /// Total WHALE tokens that are yet to be claimed by the users
    pub unclaimed_tokens: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserInfo {
    /// Total WHALE airdrop tokens claimed by the user
    pub claimed_amount: Uint128,
    /// Timestamp when the airdrop was claimed by the user
    pub timestamp: u64,
}

impl Default for UserInfo {
    fn default() -> Self {
        UserInfo {
            claimed_amount: Uint128::zero(),
            timestamp: 0u64,
        }
    }
}
