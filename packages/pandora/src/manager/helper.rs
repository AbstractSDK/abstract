use cosmwasm_std::{to_binary, CosmosMsg, Empty, StdResult, WasmMsg};

use crate::manager::msg::ExecuteMsg::UpdateModuleAddresses;

/// Register the module on the manager
/// can only be called by admin of manager
/// Factory on init
pub fn register_module_on_manager(
    manager_address: String,
    module_name: String,
    module_address: String,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: manager_address,
        msg: to_binary(&UpdateModuleAddresses {
            to_add: vec![(module_name, module_address)],
            to_remove: vec![],
        })?,
        funds: vec![],
    }))
}
