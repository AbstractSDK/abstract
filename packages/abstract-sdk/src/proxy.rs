use abstract_os::proxy::ExecuteMsg;
use cosmwasm_std::{to_binary, Addr, CosmosMsg, Empty, StdResult, WasmMsg};

/// Constructs the proxy dapp action message used by all modules.
pub fn send_to_proxy(msgs: Vec<CosmosMsg>, proxy_address: &Addr) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        msg: to_binary(&ExecuteMsg::ModuleAction { msgs })?,
        funds: vec![],
    }))
}
