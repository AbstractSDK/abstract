use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Used by Module Factory to instantiate dApp
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BaseInstantiateMsg {
    pub memory_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BaseExecuteMsg {
    /// Updates the base config
    /// Sets new values for the provided options
    UpdateConfig { treasury_address: Option<String> },
    /// Adds/removes traders
    /// If a trader is both in to_add and to_remove, it will be removed.
    UpdateTraders {
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
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
    pub traders: Vec<String>,
    pub memory_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// Template executemsg of dapp
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
}
