use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddOnInstantiateMsg {
    /// Used by Module Factory to instantiate AddOn
    pub memory_address: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AddOnExecuteMsg {
    /// Updates the base config
    UpdateConfig { memory_address: Option<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AddOnQueryMsg {
    /// Returns the base configuration for the AddOn
    Config {},
    /// Returns the admin = manager
    Admin {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddOnMigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddOnConfigResponse {
    pub proxy_address: Addr,
    pub memory_address: Addr,
    pub manager_address: Addr,
}
