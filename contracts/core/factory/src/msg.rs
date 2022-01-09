use cosmwasm_std::Addr;
use dao_os::governance::gov_type::GovernanceDetails;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Pair contract code ID, which is used to
    pub treasury_code_id: u64,
    pub token_weighted_gov_code_id: u64,
    pub memory_contract: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// UpdateConfig update relevant code IDs
    UpdateConfig {
        owner: Option<String>,
        memory_contract: Option<Addr>,
        treasury_code_id: Option<u64>,
        token_weighted_gov_code_id: Option<u64>,
    },
    /// Creates the core contracts for the OS
    CreateOS {
        /// Asset infos
        goverance: GovernanceDetails,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub memory_contract: String,
    pub treasury_code_id: u64,
    pub token_weighted_gov_code_id: u64,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
