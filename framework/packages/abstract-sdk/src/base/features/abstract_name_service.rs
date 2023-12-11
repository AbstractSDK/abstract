/// ANCHOR: ans
use crate::{ans_resolve::Resolve, cw_helpers::wasm_smart_query, AbstractSdkResult};
use abstract_core::{
    ans_host::{AssetPairingFilter, AssetPairingMapEntry, PoolAddressListResponse, QueryMsg},
    objects::{ans_host::AnsHost, DexAssetPairing},
};

use super::DepsAccess;

/// Accessor to the Abstract Name Service.
pub trait AbstractNameService: DepsAccess + Sized {
    /// Get the ANS host address.
    fn ans_host(&self) -> AbstractSdkResult<AnsHost>;

    /// Construct the name service client.
    fn name_service(&self) -> AbstractNameServiceClient<Self> {
        AbstractNameServiceClient {
            base: self,
            host: self.ans_host().unwrap(),
        }
    }
}

/// ANCHOR_END: ans

#[derive(Clone)]
pub struct AbstractNameServiceClient<'a, T: AbstractNameService> {
    base: &'a T,
    pub host: AnsHost,
}

impl<'a, T: AbstractNameService> AbstractNameServiceClient<'a, T> {
    pub fn query<R: Resolve>(&self, entry: &R) -> AbstractSdkResult<R::Output> {
        entry.resolve(&self.base.deps().querier, &self.host)
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
        let query = wasm_smart_query(
            &self.host.address,
            &QueryMsg::PoolList {
                filter,
                start_after,
                limit: page_limit,
            },
        )?;
        let resp: PoolAddressListResponse = self.base.deps().querier.query(&query)?;
        Ok(resp.pools)
    }
}
