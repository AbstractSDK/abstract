use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

use abstract_os::api::{ApiConfigResponse, BaseQueryMsg, QueryMsg, TradersResponse};
use serde::{de::DeserializeOwned, Serialize};

use crate::{state::ApiContract, ApiError};

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
        msg: QueryMsg<Q>,
        custom_query_handler: ApiQueryHandlerFn<Q, QueryError>,
    ) -> Result<Binary, QueryError> {
        match msg {
            QueryMsg::Api(api_query) => custom_query_handler
                .map(|func| func(deps, env, api_query))
                .transpose()?
                .ok_or_else(|| ApiError::NoCustomQueries {}.into()),
            QueryMsg::Base(base_query) => self.query(deps, env, base_query).map_err(From::from),
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
                to_binary(&TradersResponse {
                    traders: traders.into_iter().collect(),
                })
            }
        }
    }

    fn dapp_config(&self, deps: Deps) -> StdResult<ApiConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        Ok(ApiConfigResponse {
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
