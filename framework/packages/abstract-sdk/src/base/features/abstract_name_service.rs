/// ANCHOR: ans
use crate::{ans_resolve::Resolve, cw_helpers::ApiQuery, AbstractSdkResult};
use abstract_core::{
    ans_host::{AssetPairingFilter, AssetPairingMapEntry, PoolAddressListResponse, QueryMsg},
    objects::{ans_host::AnsHost, DexAssetPairing},
};
use cosmwasm_std::Deps;

use super::ModuleIdentification;
use crate::apis::{AbstractApi, ApiIdentification};

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
    /// Abstract Name Service Contract
    pub host: AnsHost,
}

impl<'a, T: ModuleIdentification + AbstractNameService> AbstractApi<T>
    for AbstractNameServiceClient<'a, T>
{
    fn base(&self) -> &T {
        self._base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

impl<'a, T: ModuleIdentification + AbstractNameService> ApiIdentification
    for AbstractNameServiceClient<'a, T>
{
    fn api_id() -> String {
        "AbstractNameServiceClient".to_owned()
    }
}

impl<'a, T: ModuleIdentification + AbstractNameService> AbstractNameServiceClient<'a, T> {
    /// Query ans entry
    pub fn query<R: Resolve>(&self, entry: &R) -> AbstractSdkResult<R::Output> {
        entry
            .resolve(&self.deps.querier, &self.host)
            .map_err(|error| self.wrap_query_error(error))
    }
    /// Get AnsHost
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
        let resp: PoolAddressListResponse = self.smart_query(
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
