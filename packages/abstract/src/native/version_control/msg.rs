use cosmwasm_std::{Addr, Uint64};
use cw2::ContractVersion;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::core::modules::ModuleInfo;
use terra_rust_script_derive::CosmWasmContract;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Call to add a new version and code-id for a module
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
    /// Call to add a new APi
    AddApi {
        module: String,
        version: String,
        address: String,
    },
    /// Remove an API
    RemoveApi {
        module: String,
        version: String,
    },
    /// Add a new OS to the deployed OSs
    /// Only Factory can call this
    AddOs {
        os_id: u32,
        manager_address: String,
        proxy_address: String,
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
    QueryEnabledModules {
        os_address: String,
    },
    /// Returns Core of OS
    QueryOsAddress {
        os_id: u32,
    },
    /// Queries contract code_id
    QueryCodeId {
        module: ModuleInfo,
    },
    /// Queries api addresses
    QueryApiAddress {
        module: ModuleInfo,
    },
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CodeIdResponse {
    pub code_id: Uint64,
    pub info: ContractVersion,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiAddrResponse {
    pub address: Addr,
    pub info: ContractVersion,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: String,
    pub factory: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
