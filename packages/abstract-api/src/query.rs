use abstract_sdk::manager::query_module_address;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdError, StdResult, Storage};

use abstract_os::api::{ApiQueryMsg, BaseQueryMsg, QueryApiConfigResponse, QueryTradersResponse};
use serde::de::DeserializeOwned;
use serde::Serialize;

use abstract_sdk::{Dependency, MemoryOperation};

use crate::state::ApiContract;
use crate::ApiError;

impl<T: Serialize + DeserializeOwned> MemoryOperation for ApiContract<'_, T> {
    fn load_memory(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}

impl<T: Serialize + DeserializeOwned> Dependency for ApiContract<'_, T> {
    fn dependency_address(&self, deps: Deps, dependency_name: &str) -> StdResult<Addr> {
        let manager_addr = &self
            .target_os
            .as_ref()
            .ok_or_else(|| StdError::generic_err(ApiError::NoTargetOS {}.to_string()))?
            .manager;
        query_module_address(deps, manager_addr, dependency_name)
    }
}

pub type ApiQueryHandlerFn<Q, QueryError> = Option<fn(Deps, Env, Q) -> Result<Binary, QueryError>>;

/// Where we dispatch the queries for the ApiContract
/// These ApiQueryMsg declarations can be found in `abstract_os::common_module::add_on_msg`
impl<'a, T: Serialize + DeserializeOwned> ApiContract<'a, T> {
    pub fn handle_query<
        Q: Serialize + DeserializeOwned,
        QueryError: From<cosmwasm_std::StdError> + From<ApiError>,
    >(
        &self,
        deps: Deps,
        env: Env,
        msg: ApiQueryMsg<Q>,
        custom_query_handler: ApiQueryHandlerFn<Q, QueryError>,
    ) -> Result<Binary, QueryError> {
        match msg {
            ApiQueryMsg::Api(api_query) => custom_query_handler
                .map(|func| func(deps, env, api_query))
                .transpose()?
                .ok_or_else(|| ApiError::NoCustomQueries {}.into()),
            ApiQueryMsg::Base(base_query) => self.query(deps, env, base_query).map_err(From::from),
        }
    }

    fn query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            BaseQueryMsg::Traders { proxy_address } => {
                let traders = self
                    .traders
                    .may_load(deps.storage, deps.api.addr_validate(&proxy_address)?)?
                    .unwrap_or_default();
                to_binary(&QueryTradersResponse {
                    traders: traders.into_iter().collect(),
                })
            }
        }
    }

    fn dapp_config(&self, deps: Deps) -> StdResult<QueryApiConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        Ok(QueryApiConfigResponse {
            version_control_address: state.version_control,
            memory_address: state.memory.address,
            dependencies: self
                .dependencies
                .iter()
                .map(|dep| dep.to_string())
                .collect(),
        })
    }
}
