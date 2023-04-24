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

/// Shortcut helper as the construction of QueryRequest::Wasm(WasmQuery::Raw {...}) can be quite verbose in contract code
pub fn wasm_raw_query<C>(
    contract_addr: impl Into<String>,
    key: &str,
) -> StdResult<QueryRequest<C>> {
    Ok(QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: contract_addr.into(),
        key: key.as_bytes().into(),
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

    #[test]
    fn test_wasm_raw_query() {
        let query = wasm_raw_query::<Empty>("contract", "key").unwrap();
        match query {
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                assert_eq!(contract_addr, "contract");
                assert_eq!(key, cosmwasm_std::Binary::from("key".as_bytes()));
            }
            _ => panic!("Unexpected query"),
        }
    }
}
