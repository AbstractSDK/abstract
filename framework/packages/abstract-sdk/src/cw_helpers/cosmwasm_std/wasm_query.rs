use cosmwasm_std::{QueryRequest, StdResult, WasmQuery};

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
    use cosmwasm_std::Empty;

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
