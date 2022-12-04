//! # AnsHost Entry
//! An entry (value) in the ans_host key-value store.

use cosmwasm_std::{Addr, QuerierWrapper, StdResult};
use cw_asset::{Asset, AssetInfo};

use abstract_os::objects::{ans_host::AnsHost, AnsAsset, AssetEntry, ChannelEntry, ContractEntry};

/// Resolve an [`AbstractNameService`](crate::base::features::AbstractNameService) entry into its value.
pub trait Resolve {
    type Output;
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> StdResult<Self::Output>;
}

impl Resolve for AssetEntry {
    type Output = AssetInfo;
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> StdResult<Self::Output> {
        ans_host.query_asset(querier, self)
    }
}

impl Resolve for ContractEntry {
    type Output = Addr;
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> StdResult<Self::Output> {
        ans_host.query_contract(querier, self)
    }
}

impl Resolve for ChannelEntry {
    type Output = String;
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> StdResult<Self::Output> {
        ans_host.query_channel(querier, self)
    }
}

impl Resolve for AnsAsset {
    type Output = Asset;

    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> StdResult<Self::Output> {
        Ok(Asset::new(
            ans_host.query_asset(querier, &self.info)?,
            self.amount,
        ))
    }
}

impl<T> Resolve for Vec<T>
where
    T: Resolve,
{
    type Output = Vec<T::Output>;

    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> StdResult<Self::Output> {
        self.iter()
            .map(|entry| entry.resolve(querier, ans_host))
            .collect()
    }
}
