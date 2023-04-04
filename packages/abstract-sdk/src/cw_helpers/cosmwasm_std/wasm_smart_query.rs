use cosmwasm_std::{to_binary, QueryRequest, StdResult, WasmQuery};
use serde::Serialize;

/// Shortcut helper as the construction of QueryRequest::Wasm(WasmQuery::Smart {...}) can be quite verbose in contract code
pub fn wasm_smart_query<C>(
    contract_addr: impl Into<String>,
    msg: &impl Serialize,
) -> StdResult<QueryRequest<C>> {
    let query_msg = to_binary(msg)?;
    Ok(QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.into(),
        msg: query_msg,
    }))
}

#[cfg(test)]
mod test {
    use super::*;
    use core::{app, app::BaseQueryMsg};
    use cosmwasm_std::Empty;

    #[test]
    fn test_wasm_smart_query() {
        let query_msg = app::QueryMsg::<Empty>::Base(BaseQueryMsg::Admin {});
        let query = wasm_smart_query::<Empty>("contract", &query_msg).unwrap();
        match query {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                assert_eq!(contract_addr, "contract");
                assert_eq!(msg, to_binary(&query_msg).unwrap());
            }
            _ => panic!("Unexpected query"),
        }
    }
}
