use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult, Storage};
use cw_controllers::AdminResponse;

use abstract_os::common_module::{
    add_on_msg::{AddOnConfigResponse, AddOnQueryMsg},
    traits::Mem,
};

use crate::state::AddOnContract;

impl Mem for AddOnContract<'_> {
    fn mem(&self, store: &dyn Storage) -> StdResult<abstract_os::native::memory::item::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}

/// Where we dispatch the queries for the AddOnContract
/// These AddOnQueryMsg declarations can be found in `abstract_os::common_module::add_on_msg`
impl<'a> AddOnContract<'a> {
    pub fn query(&self, deps: Deps, _env: Env, query: AddOnQueryMsg) -> StdResult<Binary> {
        match query {
            AddOnQueryMsg::Config {} => to_binary(&self.dapp_config(deps)?),
            AddOnQueryMsg::Admin {} => to_binary(&self.admin(deps)?),
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
