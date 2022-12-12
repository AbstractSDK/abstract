use cosmwasm_std::{
    Empty, Response,
};

use abstract_extension::{export_endpoints, ExtensionContract};
use abstract_sdk::os::{
    dex::{DexQueryMsg, DexRequestMsg},
    EXCHANGE,
};

use crate::{error::DexError, handlers};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexExtension = ExtensionContract<DexError, DexRequestMsg, Empty, DexQueryMsg>;
pub type DexResult = Result<Response, DexError>;

pub const DEX_EXTENSION: DexExtension = DexExtension::new(EXCHANGE, CONTRACT_VERSION)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler);

// Fails in integration testing, hency why it's commented out
// // don't export endpoints when imported as library
// #[cfg(not(feature = "library"))]
// Export the endpoints for this contract
export_endpoints!(DEX_EXTENSION, DexExtension);
