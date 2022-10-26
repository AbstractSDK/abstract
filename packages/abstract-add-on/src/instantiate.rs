use abstract_os::{
    add_on::BaseInstantiateMsg,
    module_factory::{ContextResponse, QueryMsg as FactoryQuery},
};
use cosmwasm_std::{
    to_binary, DepsMut, Env, MessageInfo, QueryRequest, StdError, StdResult, WasmQuery,
};

use abstract_sdk::memory::Memory;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    state::{AddOnContract, AddOnState},
    AddOnError,
};

use cw2::set_contract_version;

impl<
        'a,
        T: Serialize + DeserializeOwned,
        C: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<AddOnError>,
    > AddOnContract<'a, T, E, C>
{
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: BaseInstantiateMsg,
        module_name: &str,
        module_version: &str,
    ) -> StdResult<Self> {
        let add_on = Self::default();
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
        add_on.base_state.save(deps.storage, &state)?;
        add_on.admin.set(deps, Some(core.manager))?;

        Ok(AddOnContract::default())
    }
}
