use cosmwasm_std::{Addr, Deps, StdResult};
use cw_asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::memory::Memory;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetEntry(String);

impl AssetEntry {
    pub fn new<T: ToString>(entry: T) -> Self {
        Self(entry.to_string().to_ascii_lowercase())
    }
    pub fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<AssetInfo> {
        memory.query_asset(deps, &self.0)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<String> for AssetEntry {
    fn from(entry: String) -> Self {
        Self::new(entry)
    }
}

impl ToString for AssetEntry {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractEntry(String);

impl ContractEntry {
    pub fn new<T: ToString>(entry: T) -> Self {
        Self(entry.to_string().to_ascii_lowercase())
    }
    pub fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Addr> {
        memory.query_contract(deps, &self.0)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ContractEntry {
    fn from(entry: String) -> Self {
        Self::new(entry)
    }
}

impl ToString for ContractEntry {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
