use cosmwasm_std::Uint64;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Call to add a new version and code-id for a module
    /// Only Admin can call this
    AddCodeId {
        module: String,
        version: String,
        code_id: u64,
    },
    /// Remove some version of a module
    RemoveCodeId {
        module: String,
        version: String,
    },
    /// Add a new OS to the deployed OSs
    /// Only Factory can call this
    AddOs {
        os_id: u32,
        os_manager_address: String,
    },
    /// Remove an OS from the deployed OSs
    RemoveOs {
        os_id: u32,
    },
    SetAdmin {
        new_admin: String,
    },
    SetFactory {
        new_factory: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Queries enabled modules of some OS
    QueryEnabledModules { os_address: String },
    /// Queries address of OS manager module
    QueryOsAddress { os_id: u32 },
    /// Queries contract code_id
    QueryCodeId { module: String, version: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CodeIdResponse {
    pub code_id: Uint64,
}
