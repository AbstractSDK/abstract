/// ANCHOR: ans
use crate::{ans_resolve::Resolve, AbstractSdkResult};
use abstract_core::{
    ans_host::{AssetPairingFilter, AssetPairingMapEntry, PoolAddressListResponse, QueryMsg},
    objects::{ans_host::AnsHost, DexAssetPairing},
};
use cosmwasm_std::Deps;

/// Accessor to the Abstract Name Service.
pub trait AbstractNameService: Sized {
    /// Get the ANS host address.
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<AnsHost>;

    /// Construct the name service client.
    fn name_service<'a>(&'a self, deps: Deps<'a>) -> AbstractNameServiceClient<Self> {
        AbstractNameServiceClient {
            _base: self,
            deps,
            host: self.ans_host(deps).unwrap(),
        }
    }
}
/// ANCHOR_END: ans

#[derive(Clone)]
pub struct AbstractNameServiceClient<'a, T: AbstractNameService> {
    _base: &'a T,
    deps: Deps<'a>,
    pub host: AnsHost,
}

impl<'a, T: AbstractNameService> AbstractNameServiceClient<'a, T> {
    pub fn query<R: Resolve>(&self, entry: &R) -> AbstractSdkResult<R::Output> {
        entry.resolve(&self.deps.querier, &self.host)
    }
    pub fn host(&self) -> &AnsHost {
        &self.host
    }
    /// Smart-query the available trading pools.
    pub fn pool_list(
        &self,
        filter: Option<AssetPairingFilter>,
        page_limit: Option<u8>,
        start_after: Option<DexAssetPairing>,
    ) -> AbstractSdkResult<Vec<AssetPairingMapEntry>> {
        let resp: PoolAddressListResponse = self.deps.querier.query_wasm_smart(
            &self.host.address,
            &QueryMsg::PoolList {
                filter,
                start_after,
                limit: page_limit,
            },
        )?;
        Ok(resp.pools)
    }
}
