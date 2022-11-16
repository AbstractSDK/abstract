//! # AnsHost Entry
//! An entry (value) in the ans_host key-value store.

use cosmwasm_std::{Addr, Deps, StdResult};
use cw_asset::AssetInfo;

use abstract_os::objects::{ans_host::AnsHost, AssetEntry, ChannelEntry, ContractEntry};

pub trait Resolve {
    type Output;
    fn resolve(&self, deps: Deps, ans_host: &AnsHost) -> StdResult<Self::Output>;
}

impl Resolve for AssetEntry {
    type Output = AssetInfo;
    fn resolve(&self, deps: Deps, ans_host: &AnsHost) -> StdResult<Self::Output> {
        ans_host.query_asset(deps, self)
    }
}

impl Resolve for ContractEntry {
    type Output = Addr;
    fn resolve(&self, deps: Deps, ans_host: &AnsHost) -> StdResult<Self::Output> {
        ans_host.query_contract(deps, self)
    }
}

impl Resolve for ChannelEntry {
    type Output = String;
    fn resolve(&self, deps: Deps, ans_host: &AnsHost) -> StdResult<Self::Output> {
        ans_host.query_channel(deps, self)
    }
}
