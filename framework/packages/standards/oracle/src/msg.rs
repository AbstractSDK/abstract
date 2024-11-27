#![warn(missing_docs)]
//! # Oracle Adapter API
// re-export response types
use abstract_std::adapter;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Empty};

/// The name of the oracle to query prices from.
pub type OracleName = String;

/// Top-level Abstract Adapter execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
pub type ExecuteMsg = adapter::ExecuteMsg<Empty>;
/// Top-level Abstract Adapter instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
pub type InstantiateMsg = adapter::InstantiateMsg<Empty>;
/// Top-level Abstract Adapter query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type QueryMsg = adapter::QueryMsg<OracleQueryMsg>;

impl adapter::AdapterQueryMsg for OracleQueryMsg {}

/// Query messages for the oracle adapter
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum OracleQueryMsg {
    /// Query the oracle adapter config
    #[returns(Config)]
    Config {},
    /// Query the latest price attached to the price source key
    #[returns(PriceResponse)]
    Price {
        /// Identifier of the oracle value that you wish to query on the oracle
        price_source_key: String,
        /// Identifier of the oracle
        oracle: OracleName,
        /// Maximum age of the price
        max_age: Seconds,
    },
}

/// Alias to document time unit the oracle adapter expects data to be in.
pub type Seconds = u64;

/// Price Response returned by an adapter query
#[cosmwasm_schema::cw_serde]
pub struct PriceResponse {
    /// Price response
    pub price: Decimal,
}

/// No Config for this adapter
#[cosmwasm_schema::cw_serde]
pub struct Config {}
