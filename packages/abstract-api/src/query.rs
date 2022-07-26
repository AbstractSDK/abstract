use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult, Storage};

use abstract_os::api::{ApiQueryMsg, QueryApiConfigResponse, QueryTradersResponse};
use serde::de::DeserializeOwned;
use serde::Serialize;

use abstract_sdk::LoadMemory;

use crate::state::ApiContract;

impl<T: Serialize + DeserializeOwned> LoadMemory for ApiContract<'_, T> {
    fn mem(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}

/// Where we dispatch the queries for the ApiContract
/// These ApiQueryMsg declarations can be found in `abstract_os::common_module::add_on_msg`
impl<'a, T: Serialize + DeserializeOwned> ApiContract<'a, T> {
    pub fn query(&self, deps: Deps, _env: Env, query: ApiQueryMsg) -> StdResult<Binary> {
        match query {
            ApiQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            ApiQueryMsg::Traders { proxy_address } => {
                let traders = self
                    .traders
                    .load(deps.storage, deps.api.addr_validate(&proxy_address)?)?;
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
