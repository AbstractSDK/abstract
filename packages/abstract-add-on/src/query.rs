use abstract_os::add_on::{AddOnQueryMsg, QueryAddOnConfigResponse};
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdError, StdResult, Storage};
use cw_controllers::AdminResponse;

use abstract_sdk::{manager::query_module_address, Dependency, MemoryOperation};

use crate::state::AddOnContract;

impl MemoryOperation for AddOnContract<'_> {
    fn load_memory(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}

impl Dependency for AddOnContract<'_> {
    fn dependency_address(&self, deps: Deps, dependency_name: &str) -> StdResult<Addr> {
        let manager_addr = &self
            .admin
            .get(deps)?
            .ok_or_else(|| StdError::generic_err("No admin on add-on"))?;
        query_module_address(deps, manager_addr, dependency_name)
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

    fn dapp_config(&self, deps: Deps) -> StdResult<QueryAddOnConfigResponse> {
        let state = self.base_state.load(deps.storage)?;
        let admin = self.admin.get(deps)?.unwrap();
        Ok(QueryAddOnConfigResponse {
            proxy_address: state.proxy_address,
            memory_address: state.memory.address,
            manager_address: admin,
        })
    }

    fn admin(&self, deps: Deps) -> StdResult<AdminResponse> {
        self.admin.query_admin(deps)
    }
}
