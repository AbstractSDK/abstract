use abstract_std::{
    ans_host::{
        AssetPairingFilter, AssetPairingMapEntry, PoolAddressListResponse, QueryMsg,
        RegisteredDexesResponse,
    },
    objects::{ans_host::AnsHost, DexAssetPairing},
};
use cosmwasm_std::{Deps, Env};

use super::ModuleIdentification;
use crate::apis::AbstractApi;
use crate::{ans_resolve::Resolve, cw_helpers::ApiQuery, AbstractSdkResult};

/// ANCHOR: ans
/// Accessor to the Abstract Name Service.
pub trait AbstractNameService: Sized {
    /// Get the ANS host address.
    fn ans_host(&self, deps: Deps, env: &Env) -> AbstractSdkResult<AnsHost>;

    /// Construct the name service client.
    fn name_service<'a>(&'a self, deps: Deps<'a>, env: &Env) -> AbstractNameServiceClient<Self> {
        AbstractNameServiceClient {
            base: self,
            deps,
            host: self.ans_host(deps, env).unwrap(),
        }
    }
}
/// ANCHOR_END: ans

#[derive(Clone)]
pub struct AbstractNameServiceClient<'a, T: AbstractNameService> {
    base: &'a T,
    deps: Deps<'a>,
    /// Abstract Name Service Contract
    pub host: AnsHost,
}

impl<'a, T: ModuleIdentification + AbstractNameService> AbstractApi<T>
    for AbstractNameServiceClient<'a, T>
{
    const API_ID: &'static str = "AbstractNameServiceClient";

    fn base(&self) -> &T {
        self.base
    }
    fn deps(&self) -> Deps {
        self.deps
    }
}

impl<'a, T: ModuleIdentification + AbstractNameService> AbstractNameServiceClient<'a, T> {
    /// Query ans entry
    pub fn query<R: Resolve>(&self, entry: &R) -> AbstractSdkResult<R::Output> {
        entry
            .resolve(&self.deps.querier, &self.host)
            .map_err(|error| self.wrap_query_error(error))
    }

    /// Returns if the entry is registered on the ANS.
    /// Will return an Err if the query failed for technical reasons (like a wrong address or state parsing error).
    /// Will return true if found.
    /// Will return false if not found.
    pub fn is_registered(&self, entry: &impl Resolve) -> bool {
        entry.is_registered(&self.deps.querier, &self.host)
    }
    /// Assert that an entry is registered on the ANS. Will return an Err if the entry is not registered.
    pub fn assert_registered(&self, entry: &impl Resolve) -> AbstractSdkResult<()> {
        entry
            .assert_registered(&self.deps.querier, &self.host)
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
            .query_registered_dexes(&self.deps.querier)
            .map_err(|error| self.wrap_query_error(error))
    }
}
