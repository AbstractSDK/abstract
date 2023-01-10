use cosmwasm_std::{Empty, Response};

use abstract_api::{ApiContract};
use abstract_sdk::os::{
    dex::{DexExecuteMsg, DexQueryMsg},
    EXCHANGE,
};

use crate::{error::DexError, handlers};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexApi = ApiContract<DexError, DexExecuteMsg, Empty, DexQueryMsg>;
pub type DexResult = Result<Response, DexError>;

pub const DEX_API: DexApi = DexApi::new(EXCHANGE, CONTRACT_VERSION, None)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

// don't export endpoints when imported as library
#[cfg(not(feature = "library"))]
export_endpoints!(DEX_API, DexApi);
