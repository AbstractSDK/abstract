use abstract_os::add_on::AddOnExecuteMsg;
use abstract_sdk::common_module::ProxyExecute;
use abstract_sdk::proxy::send_to_proxy;
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};

use crate::error::AddOnError;
use crate::state::AddOnContract;
use crate::AddOnResult;

impl ProxyExecute for AddOnContract<'_> {
    type Err = AddOnError;
    fn execute_on_proxy(
        &self,
        deps: Deps,
        msgs: Vec<cosmwasm_std::CosmosMsg>,
    ) -> Result<Response, Self::Err> {
        let proxy = self.base_state.load(deps.storage)?.proxy_address;
        Ok(Response::new().add_message(send_to_proxy(msgs, &proxy)?))
    }
}

impl<'a> AddOnContract<'a> {
    pub fn execute(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: AddOnExecuteMsg,
    ) -> AddOnResult {
        match message {
            AddOnExecuteMsg::UpdateConfig { memory_address } => {
                self.update_config(deps, info, memory_address)
            }
        }
    }

    fn update_config(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        memory_address: Option<String>,
    ) -> AddOnResult {
        // self._update_config(deps, info, memory_address)?;
        // Only the admin should be able to call this
        self.admin.assert_admin(deps.as_ref(), &info.sender)?;

        let mut state = self.base_state.load(deps.storage)?;

        if let Some(memory_address) = memory_address {
            state.memory.address = deps.api.addr_validate(memory_address.as_str())?;
        }

        self.base_state.save(deps.storage, &state)?;

        Ok(Response::default().add_attribute("action", "updated_memory_address"))
    }
}
