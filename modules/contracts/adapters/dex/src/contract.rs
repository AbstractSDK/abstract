use crate::handlers;
use crate::EXCHANGE;

use crate::msg::{DexExecuteMsg, DexInstantiateMsg, DexQueryMsg};
use abstract_adapter::{export_endpoints, AdapterContract};
use abstract_dex_adapter_traits::DexError;
use cosmwasm_std::Response;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexAdapter = AdapterContract<DexError, DexInstantiateMsg, DexExecuteMsg, DexQueryMsg>;
pub type DexResult<T = Response> = Result<T, DexError>;

pub const DEX_ADAPTER: DexAdapter = DexAdapter::new(EXCHANGE, CONTRACT_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

#[cfg(feature = "export")]
export_endpoints!(DEX_ADAPTER, DexAdapter);
