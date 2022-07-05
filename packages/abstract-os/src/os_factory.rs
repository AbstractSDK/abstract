//! # Memory
//!
//! `abstract_os::memory` stores chain-specific contract addresses.
//!
//! ## Description
//! Contract and asset addresses are stored on the proxy contract and are retrievable trough smart or raw queries.
//! This is useful when managing a large set of contracts.

use crate::objects::gov_type::GovernanceDetails;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Version control contract used to get code-ids and register OS
    pub version_control_address: String,
    /// Memory contract
    pub memory_address: String,
    pub module_factory_address: String,
    /// Provide the deployment chain-id
    pub chain_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    /// Update config
    UpdateConfig {
        admin: Option<String>,
        memory_contract: Option<String>,
        version_control_contract: Option<String>,
        module_factory_address: Option<String>,
        subscription_address: Option<String>,
    },
    /// Creates the core contracts for the OS
    CreateOs {
        /// Governance details
        /// TODO: add support for other types of gov.
        governance: GovernanceDetails,
        os_name: String,
        description: Option<String>,
        link: Option<String>,
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
    pub version_control_contract: String,
    pub module_factory_address: String,
    pub subscription_address: Option<String>,
    pub next_os_id: u32,
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
