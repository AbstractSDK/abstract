use abstract_os::add_on::{AddOnConfigResponse, BaseQueryMsg};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};
use cw_controllers::AdminResponse;
use serde::{de::DeserializeOwned, Serialize};

use crate::{state::AddOnContract, AddOnError};

/// Where we dispatch the queries for the AddOnContract
/// These BaseQueryMsg declarations can be found in `abstract_os::common_module::add_on_msg`
impl<
        'a,
        T: Serialize + DeserializeOwned,
        C: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<AddOnError>,
    > AddOnContract<'a, T, E, C>
{
    pub fn query(&self, deps: Deps, _env: Env, query: BaseQueryMsg) -> StdResult<Binary> {
        match query {
            BaseQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            BaseQueryMsg::Admin {} => to_binary(&self.admin(deps)?),
        }
    }

    fn dapp_config(&self, deps: Deps) -> StdResult<AddOnConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        let admin = self.admin.get(deps)?.unwrap();
        Ok(AddOnConfigResponse {
            proxy_address: state.proxy_address,
            memory_address: state.memory.address,
            manager_address: admin,
        })
    }

    fn admin(&self, deps: Deps) -> StdResult<AdminResponse> {
        self.admin.query_admin(deps)
    }
}
