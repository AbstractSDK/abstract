use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiInstantiateMsg {
    /// Used to easily perform queries
    pub memory_address: String,
    /// Used to verify senders
    pub version_control_address: String,
}

/// Api request forwards generated msg to the optionally attached proxy addr.
/// If proxy = None, then the sender must be an OS manager.  
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiRequestMsg<T> {
    // Ok to assume address is validated as all map keys are previously validated
    pub proxy_addr: Option<Addr>,
    // The actual request
    pub request: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ApiExecuteMsg {
    /// Adds/removes traders
    /// If a trader is both in to_add and to_remove, it will be removed.
    UpdateTraders {
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    },
}

/// Rename to ApiQueryMsg
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiQueryMsg {
    /// Returns the base configuration for the DApp
    Config {},

    /// Return type: TradersResponse.
    /// TODO: enable pagination of some sort
    Traders { proxy_addr: String },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ApiConfigResponse {
    pub version_control_address: Addr,
    pub memory_address: Addr,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct TradersResponse {
    /// Contains all traders
    pub traders: Vec<Addr>,
}
