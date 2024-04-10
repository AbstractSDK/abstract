use crate::{
    handlers,
    msg::{OracleExecuteMsg, OracleInstantiateMsg, OracleQueryMsg},
    ORACLE_ADAPTER_ID,
};

use abstract_adapter::{export_endpoints, AdapterContract};
use abstract_oracle_standard::OracleError;
use cosmwasm_std::Response;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type OracleAdapter =
    AdapterContract<OracleError, OracleInstantiateMsg, OracleExecuteMsg, OracleQueryMsg>;
pub type OracleResult<T = Response> = Result<T, OracleError>;

pub const ORACLE_ADAPTER: OracleAdapter =
    OracleAdapter::new(ORACLE_ADAPTER_ID, CONTRACT_VERSION, None)
        .with_instantiate(handlers::instantiate_handler)
        .with_execute(handlers::execute_handler)
        .with_query(handlers::query_handler);

#[cfg(feature = "export")]
export_endpoints!(ORACLE_ADAPTER, OracleAdapter);
