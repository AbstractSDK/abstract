use abstract_os::ibc_host::BaseInstantiateMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, StdResult};
use serde::{de::DeserializeOwned, Serialize};

use abstract_sdk::memory::Memory;

use crate::state::{Host, HostState, CLOSED_CHANNELS};

use cw2::set_contract_version;

impl<'a, T: Serialize + DeserializeOwned> Host<'a, T> {
    /// Instantiate the API
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: BaseInstantiateMsg,
        module_name: &str,
        module_version: &str,
        chain: &str,
    ) -> StdResult<Self> {
        let api = Self::new(&[]);
        let memory = Memory {
            address: deps.api.addr_validate(&msg.memory_address)?,
        };

        // Base state
        let state = HostState {
            chain: chain.to_string(),
            memory,
            cw1_code_id: msg.cw1_code_id,
            admin: info.sender,
        };
        // Keep track of all the closed channels, allows for fund recovery if channel closes.
        let closed_channels = vec![];
        CLOSED_CHANNELS.save(deps.storage, &closed_channels)?;
        set_contract_version(deps.storage, module_name, module_version)?;
        api.base_state.save(deps.storage, &state)?;

        Ok(api)
    }
}
