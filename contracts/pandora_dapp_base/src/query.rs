use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw_controllers::AdminResponse;
use serde::de::DeserializeOwned;
use serde::Serialize;

use pandora_os::pandora_dapp::msg::DappQueryMsg;
use pandora_os::pandora_dapp::query::{DappStateResponse, TradersResponse};
use pandora_os::pandora_dapp::traits::{CustomMsg, DappQuery};

use crate::state::DappContract;

/// Where we dispatch the queries for the DappContract
/// These DappQueryMsg declarations can be found in `msg`
impl<'a, T, C> DappContract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    pub fn query(&self, deps: Deps, _env: Env, query: DappQueryMsg) -> StdResult<Binary> {
        match query {
            DappQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            DappQueryMsg::Traders { /* start_after, limit */ } => {
                to_binary(&self.all_traders(deps)?)
            }
            DappQueryMsg::Admin {} => to_binary(&self.admin(deps)?),
        }
    }
}

/// Where the actual querying methods themselves are defined
/// Their interfaces are declared in `traits`
impl<'a, T, C> DappQuery<T> for DappContract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    fn dapp_config(&self, deps: Deps) -> StdResult<DappStateResponse> {
        let state = self.base_state.load(deps.storage)?;

        Ok(DappStateResponse {
            proxy_address: state.proxy_address,
            memory_address: state.memory.address,
            traders: state.traders,
        })
    }

    fn admin(&self, deps: Deps) -> StdResult<AdminResponse> {
        self.admin.query_admin(deps)
    }

    /// TODO: enable pagination
    fn all_traders(
        &self,
        deps: Deps,
        // _start_after: Option<String>,
        // _limit: Option<u32>,
    ) -> StdResult<TradersResponse> {
        let state = self.base_state.load(deps.storage)?;

        Ok(TradersResponse {
            traders: state.traders,
        })
    }
}
