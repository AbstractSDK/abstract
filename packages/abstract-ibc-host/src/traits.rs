use abstract_sdk::{MemoryOperation, OsExecute};
use cosmwasm_std::{wasm_execute, Deps, StdError, StdResult, Storage, SubMsg};

use crate::{Host, HostError};

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > MemoryOperation
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn load_memory(&self, store: &dyn Storage) -> StdResult<abstract_sdk::memory::Memory> {
        Ok(self.base_state.load(store)?.memory)
    }
}

/// Execute a set of CosmosMsgs on the proxy contract of an OS.
impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > OsExecute
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn os_execute(
        &self,
        _deps: Deps,
        msgs: Vec<cosmwasm_std::CosmosMsg>,
    ) -> Result<SubMsg, StdError> {
        if let Some(target) = &self.proxy_address {
            let reflect_msg = cw1_whitelist::msg::ExecuteMsg::Execute { msgs };
            let wasm_msg = wasm_execute(target, &reflect_msg, vec![])?;
            Ok(SubMsg::new(wasm_msg))
        } else {
            Err(StdError::generic_err(HostError::NoTarget.to_string()))
        }
    }
    fn os_ibc_execute(
        &self,
        _deps: Deps,
        _msgs: Vec<abstract_os::ibc_client::ExecuteMsg>,
    ) -> Result<SubMsg, StdError> {
        Err(StdError::generic_err(HostError::IbcHopping.to_string()))
    }
}
