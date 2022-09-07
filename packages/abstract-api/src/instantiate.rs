use abstract_os::api::ApiInstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, StdResult};
use serde::de::DeserializeOwned;
use serde::Serialize;

use abstract_sdk::memory::Memory;

use crate::state::{ApiContract, ApiState};

use cw2::set_contract_version;

impl<'a, T: Serialize + DeserializeOwned> ApiContract<'a, T> {
    /// Instantiate the API
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: ApiInstantiateMsg,
        module_name: &str,
        module_version: &str,
        _api_dependencies: Vec<String>,
    ) -> StdResult<Self> {
        let api = Self::default();
        let memory = Memory {
            address: deps.api.addr_validate(&msg.memory_address)?,
        };

        // Base state
        let state = ApiState {
            version_control: deps.api.addr_validate(&msg.version_control_address)?,
            memory,
        };

        set_contract_version(deps.storage, module_name, module_version)?;
        api.base_state.save(deps.storage, &state)?;

        Ok(api)
    }
}
