use cosmwasm_std::{StdResult, Addr, Deps};
use cw_asset::AssetInfo;

use super::memory::Memory;

pub struct AssetEntry(String);

impl AssetEntry {
    pub fn new<T: ToString>(entry: T) -> Self {
        Self(entry.to_string().to_ascii_lowercase())
    }
    pub fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<AssetInfo> {
        memory.query_asset(deps, &self.0)
    }
}
impl From<String> for AssetEntry {
    fn from(entry: String) -> Self {
        Self::new(entry)
    }
}
pub struct ContractEntry(String);

impl ContractEntry {
    pub fn new<T: ToString>(entry: T) -> Self {
        Self(entry.to_string().to_ascii_lowercase())
    }
    pub fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Addr> {
        memory.query_contract(deps, &self.0)
    }
}

impl From<String> for ContractEntry {
    fn from(entry: String) -> Self {
        Self::new(entry)
    }
}
