use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BaseInstantiateMsg {
    pub treasury_address: String,
    pub trader: String,
    pub memory_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaseExecuteMsg {
    /// Updates the base config
    UpdateConfig {
        treasury_address: Option<String>,
        trader: Option<String>,
        memory: Option<String>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaseQueryMsg {
    /// Returns the state of the DApp
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BaseStateResponse {
    pub treasury_address: String,
    pub trader: String,
    pub memory_address: String,
}
