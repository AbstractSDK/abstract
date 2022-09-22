use abstract_os::add_on::BaseExecuteMsg;

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{state::AddOnContract, AddOnResult};

impl<'a> AddOnContract<'a> {
    pub fn execute(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> AddOnResult {
        match message {
            BaseExecuteMsg::UpdateConfig { memory_address } => {
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
