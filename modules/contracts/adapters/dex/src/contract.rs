use abstract_adapter::AdapterContract;
use abstract_dex_standard::{
    msg::{DexExecuteMsg, DexInstantiateMsg, DexQueryMsg},
    DexError,
};
use cosmwasm_std::Response;

use crate::{handlers, DEX_ADAPTER_ID};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexAdapter = AdapterContract<DexError, DexInstantiateMsg, DexExecuteMsg, DexQueryMsg>;
pub type DexResult<T = Response> = Result<T, DexError>;

pub const DEX_ADAPTER: DexAdapter = DexAdapter::new(DEX_ADAPTER_ID, CONTRACT_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

#[cfg(feature = "export")]
use abstract_adapter::export_endpoints;
#[cfg(feature = "export")]
export_endpoints!(DEX_ADAPTER, DexAdapter);
