use cosmwasm_std::{to_binary, Addr, Deps, QueryRequest, StdResult, Uint128, WasmQuery};

use crate::core::treasury::msg::{QueryMsg, TotalValueResponse};

/// Query the total value denominated in the vault base asset
/// The provided address must implement the TotalValue Query
pub fn query_total_value(deps: Deps, vault_address: &Addr) -> StdResult<Uint128> {
    let response: TotalValueResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: vault_address.to_string(),
            msg: to_binary(&QueryMsg::TotalValue {})?,
        }))?;

    Ok(response.value)
}
