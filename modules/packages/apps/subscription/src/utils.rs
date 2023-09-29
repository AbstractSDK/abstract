use abstract_sdk::core::manager::ExecuteMsg as ManagerMsg;
use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdResult, WasmMsg};

pub fn suspend_os(manager_address: Addr, new_suspend_status: bool) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: manager_address.to_string(),
        msg: to_binary(&ManagerMsg::UpdateStatus {
            is_suspended: Some(new_suspend_status),
        })?,
        funds: vec![],
    }))
}
