use crate::handlers;
use crate::DEX_ADAPTER_ID;

use abstract_adapter::{export_endpoints, AdapterContract};
use abstract_dex_standard::msg::{DexExecuteMsg, DexInstantiateMsg, DexQueryMsg};
use abstract_dex_standard::DexError;
use cosmwasm_std::Response;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexAdapter = AdapterContract<DexError, DexInstantiateMsg, DexExecuteMsg, DexQueryMsg>;
pub type DexResult<T = Response> = Result<T, DexError>;

pub const DEX_ADAPTER: DexAdapter = DexAdapter::new(DEX_ADAPTER_ID, CONTRACT_VERSION, None)
    .instantiate(handlers::instantiate_handler)
    .execute(handlers::execute_handler)
    .query(handlers::query_handler);

#[cfg(feature = "export")]
export_endpoints!(DEX_ADAPTER, DexAdapter);
