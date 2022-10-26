//! # Memory Entry
//! An entry (value) in the memory key-value store.

use cosmwasm_std::{Addr, Deps, StdResult};
use cw_asset::AssetInfo;

use abstract_os::objects::{memory::Memory, AssetEntry, ChannelEntry, ContractEntry};

pub trait Resolve {
    type Output;
    fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Self::Output>;
}

impl Resolve for AssetEntry {
    type Output = AssetInfo;
    fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Self::Output> {
        memory.query_asset(deps, self)
    }
}

impl Resolve for ContractEntry {
    type Output = Addr;
    fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Self::Output> {
        memory.query_contract(deps, self)
    }
}

impl Resolve for ChannelEntry {
    type Output = String;
    fn resolve(&self, deps: Deps, memory: &Memory) -> StdResult<Self::Output> {
        memory.query_channel(deps, self)
    }
}
