use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult};
use serde::de::DeserializeOwned;
use serde::Serialize;

use abstract_os::common_module::api_msg::ApiInstantiateMsg;
use abstract_os::native::memory::item::Memory;

use crate::state::{ApiContract, ApiState};

use cw2::set_contract_version;

impl<'a, T: Serialize + DeserializeOwned> ApiContract<'a, T> {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: ApiInstantiateMsg,
        module_name: &str,
        module_version: &str,
    ) -> StdResult<Response> {
        let memory = Memory {
            address: deps.api.addr_validate(&msg.memory_address)?,
        };

        // Base state
        let state = ApiState {
            version_control: deps.api.addr_validate(&msg.version_control_address)?,
            memory,
        };

        set_contract_version(deps.storage, module_name, module_version)?;
        self.base_state.save(deps.storage, &state)?;

        Ok(Response::default())
    }
}
