use abstract_os::api::BaseInstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, StdResult};
use serde::{de::DeserializeOwned, Serialize};

use abstract_sdk::memory::Memory;

use crate::{
    state::{ApiContract, ApiState},
    ApiError,
};

use cw2::set_contract_version;

impl<'a, T: Serialize + DeserializeOwned, E: From<cosmwasm_std::StdError> + From<ApiError>>
    ApiContract<'a, T, E>
{
    /// Instantiate the API
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: BaseInstantiateMsg,
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
