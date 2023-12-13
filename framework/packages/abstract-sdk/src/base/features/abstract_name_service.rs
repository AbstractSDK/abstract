/// ANCHOR: ans
use crate::{ans_resolve::Resolve, cw_helpers::ApiQuery, AbstractSdkResult};
use abstract_core::{
    ans_host::{
        AssetPairingFilter, AssetPairingMapEntry, PoolAddressListResponse, QueryMsg,
        RegisteredDexesResponse,
    },
    objects::{ans_host::AnsHost, DexAssetPairing},
};

use super::DepsAccess;

use super::ModuleIdentification;
use crate::apis::{AbstractApi, ApiIdentification};

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
    /// Abstract Name Service Contract
    pub host: AnsHost,
}

impl<'a, T: ModuleIdentification + AbstractNameService> AbstractApi<T>
    for AbstractNameServiceClient<'a, T>
{
    fn base(&self) -> &T {
        self.base
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
            .resolve(&self.base.deps().querier, &self.host)
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
    /// Raw-query the available dexes on the chain.
    pub fn registered_dexes(&self) -> AbstractSdkResult<RegisteredDexesResponse> {
        self.host
            .query_registered_dexes(&self.base.deps().querier)
            .map_err(|error| self.wrap_query_error(error))
    }
}
