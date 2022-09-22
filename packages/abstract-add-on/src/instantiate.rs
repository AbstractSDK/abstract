use abstract_os::{
    add_on::BaseInstantiateMsg,
    module_factory::{ContextResponse, QueryMsg as FactoryQuery},
};
use cosmwasm_std::{
    to_binary, DepsMut, Env, MessageInfo, QueryRequest, StdError, StdResult, WasmQuery,
};

use abstract_sdk::memory::Memory;

use crate::state::{AddOnContract, AddOnState};

use cw2::set_contract_version;

impl<'a> AddOnContract<'a> {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: BaseInstantiateMsg,
        module_name: &str,
        module_version: &str,
    ) -> StdResult<Self> {
        let memory = Memory {
            address: deps.api.addr_validate(&msg.memory_address)?,
        };

        // Caller is factory so get proxy and manager (admin) from there
        let resp: ContextResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: info.sender.to_string(),
            msg: to_binary(&FactoryQuery::Context {})?,
        }))?;

        let core = match resp.core {
            Some(core) => core,
            None => {
                return Err(StdError::generic_err(
                    "context of module factory not properly set.",
                ))
            }
        };

        // Base state
        let state = AddOnState {
            proxy_address: core.proxy.clone(),
            memory,
        };

        set_contract_version(deps.storage, module_name, module_version)?;
        self.base_state.save(deps.storage, &state)?;
        self.admin.set(deps, Some(core.manager))?;

        Ok(AddOnContract::default())
    }
}
