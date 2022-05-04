use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DappInstantiateMsg {
    /// Used by Module Factory to instantiate dApp
    pub memory_address: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DappExecuteMsg {
    /// Updates the base config
    /// Sets new values for the provided options
    UpdateConfig { proxy_address: Option<String> },
    /// Adds/removes traders
    /// If a trader is both in to_add and to_remove, it will be removed.
    UpdateTraders {
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

/// Rename to DappQueryMsg
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DappQueryMsg {
    /// Returns the base configuration for the DApp
    Config {},

    /// Return type: TradersResponse.
    /// TODO: enable pagination of some sort
    Traders {
        // start_after: Option<String>,
    // limit: Option<u32>,
    },

    /// Returns the admin
    Admin {},
}
