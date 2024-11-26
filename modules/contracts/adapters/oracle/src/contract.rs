use abstract_adapter::AdapterContract;
use abstract_oracle_standard::{msg::OracleQueryMsg, OracleError};
use cosmwasm_std::{Empty, Response};

use crate::{handlers, ORACLE_ADAPTER_ID};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type OracleAdapter = AdapterContract<OracleError, Empty, Empty, OracleQueryMsg>;
pub type OracleResult<T = Response> = Result<T, OracleError>;

pub const ORACLE_ADAPTER: OracleAdapter =
    OracleAdapter::new(ORACLE_ADAPTER_ID, CONTRACT_VERSION, None)
        .with_instantiate(handlers::instantiate_handler)
        .with_execute(handlers::execute_handler)
        .with_query(handlers::query_handler);

#[cfg(feature = "export")]
use abstract_adapter::export_endpoints;
#[cfg(feature = "export")]
export_endpoints!(ORACLE_ADAPTER, OracleAdapter);
