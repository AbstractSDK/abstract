use cosmwasm_std::{
    to_binary, Addr, Deps, QuerierWrapper, QueryRequest, StdResult, Uint128, WasmQuery,
};
use cw20::{Cw20QueryMsg, TokenInfoResponse};

use abstract_os::proxy::{QueryMsg, TotalValueResponse};

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

pub fn query_supply(querier: &QuerierWrapper, contract_addr: Addr) -> StdResult<Uint128> {
    let res: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: String::from(contract_addr),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(res.total_supply)
}
