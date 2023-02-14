use crate::{error::DexError, handlers};
use abstract_api::ApiContract;
use abstract_sdk::os::{
    dex::{DexExecuteMsg, DexQueryMsg},
    EXCHANGE,
};
use cosmwasm_std::{Empty, Response};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexApi = ApiContract<DexError, DexExecuteMsg, Empty, DexQueryMsg>;
pub type DexResult<T = Response> = Result<T, DexError>;

pub const DEX_API: DexApi = DexApi::new(EXCHANGE, CONTRACT_VERSION, None)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

#[cfg(not(feature = "library"))]
abstract_api::export_endpoints!(DEX_API, DexApi);
