use abstract_std::AbstractError;
use cosmwasm_std::{Binary, QueryRequest, StdResult, WasmQuery};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    apis::{AbstractApi, ApiIdentification},
    features::ModuleIdentification,
    AbstractSdkError, AbstractSdkResult,
};

/// Shortcut helper as the construction of QueryRequest::Wasm(WasmQuery::Raw {...}) can be quite verbose in contract code
pub fn wasm_raw_query<C>(
    contract_addr: impl Into<String>,
    key: impl Into<Binary>,
) -> StdResult<QueryRequest<C>> {
    Ok(QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: contract_addr.into(),
        key: key.into(),
    }))
}

pub trait ApiQuery<S: ModuleIdentification>: AbstractApi<S> + ApiIdentification {
    fn smart_query<T: DeserializeOwned>(
        &self,
        contract_addr: impl Into<String>,
        msg: &impl Serialize,
    ) -> AbstractSdkResult<T> {
        let querier = self.deps().querier;
        querier
            .query_wasm_smart(contract_addr, msg)
            .map_err(|error| self.wrap_query_error(error))
    }

    fn wrap_query_error(&self, error: impl Into<AbstractError>) -> AbstractSdkError {
        AbstractSdkError::ApiQuery {
            api: Self::api_id(),
            module_id: self.base().module_id().to_owned(),
            error: Box::new(error.into()),
        }
    }
}

impl<S, T> ApiQuery<S> for T
where
    S: ModuleIdentification,
    T: AbstractApi<S> + ApiIdentification,
{
}

#[cfg(test)]
mod test {
    use cosmwasm_std::Empty;
    use cw_storage_plus::Path;

    use super::*;

    #[test]
    fn test_wasm_raw_query() {
        let query = wasm_raw_query::<Empty>("contract", b"key").unwrap();
        match query {
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                assert_eq!(contract_addr, "contract");
                assert_eq!(key, cosmwasm_std::Binary::from(b"key"));
            }
            _ => panic!("Unexpected query"),
        }
    }

    #[test]
    fn test_wasm_raw_map_query() {
        let key: Path<u64> = Path::new(b"map", &[&4u8.to_be_bytes()]);
        println!("p: {}", String::from_utf8(key.to_vec()).unwrap());
        let query = wasm_raw_query::<Empty>("contract", key.to_vec()).unwrap();
        match query {
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                assert_eq!(contract_addr, "contract");
                // namespace length, namespace, key
                assert_eq!(key, cosmwasm_std::Binary::from(b"\x00\x03map\x04"));
            }
            _ => panic!("Unexpected query"),
        }
    }
}
