use abstract_sdk::common_namespace::BASE_STATE_KEY;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult, Storage};

use abstract_os::api::{ApiQueryMsg, BaseQueryMsg, QueryApiConfigResponse, QueryTradersResponse};
use serde::de::DeserializeOwned;
use serde::Serialize;

use abstract_sdk::MemoryOperation;

use crate::state::{ApiContract, TRADER_NAMESPACE};
use crate::ApiError;

impl<T: Serialize + DeserializeOwned> MemoryOperation for ApiContract<'_, T> {
    fn load_memory(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
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
            ApiQueryMsg::Base(base_query) => {
                let api = Self::new(BASE_STATE_KEY, TRADER_NAMESPACE, Addr::unchecked(""));
                api.query(deps, env, base_query).map_err(From::from)
            }
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
            dependencies: state.api_dependencies,
        })
    }
}
