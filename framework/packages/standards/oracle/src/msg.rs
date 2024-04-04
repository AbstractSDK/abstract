#![warn(missing_docs)]
//! # Oracle Adapter API
// re-export response types
use abstract_core::{
    adapter,
    objects::{price_source::UncheckedPriceSource, AssetEntry},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Uint128;
use cw_asset::AssetInfo;

pub use crate::action::OracleAction;
use crate::state::{OraclePriceSource, TotalValue};

/// The name of the oracle to trade on.
pub type ProviderName = String;

/// Top-level Abstract Adapter execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
pub type ExecuteMsg = adapter::ExecuteMsg<OracleExecuteMsg>;
/// Top-level Abstract Adapter instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
pub type InstantiateMsg = adapter::InstantiateMsg<OracleInstantiateMsg>;
/// Top-level Abstract Adapter query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type QueryMsg = adapter::QueryMsg<OracleQueryMsg>;

impl adapter::AdapterExecuteMsg for OracleExecuteMsg {}
impl adapter::AdapterQueryMsg for OracleQueryMsg {}

/// Instantiation message for oracle adapter
#[cosmwasm_schema::cw_serde]
pub struct OracleInstantiateMsg {
    /// Maximum age of external price source before getting filtrated in calculations
    pub external_age_max: u64,
    /// Addresses of providers
    pub providers: Vec<(ProviderName, String)>,
}

/// Oracle Execute msg
#[cosmwasm_schema::cw_serde]
pub enum OracleExecuteMsg {
    /// Admin action to perform on the Oracle adapter
    /// This can be done only by oracle admin(abstract namespace owner) and saved state will be used for default values during queries
    Admin(OracleAction),
    /// Action to perform on the Oracle adapter
    Account(OracleAction),
    // TODO: update provider_addrs
}

// /// Query messages for the oracle adapter
// #[cosmwasm_schema::cw_serde]
// #[derive(QueryResponses)]
// #[query_responses(nested)]
// pub enum OracleQueryMsg {
//     /// Query for arbitrary address
//     /// Default values provided by adapter owner used for calculations
//     Address(AddressQueryMsg),
//     /// Query for proxy address
//     Account(AccountQueryMsg),
// }

// #[cosmwasm_schema::cw_serde]
// #[derive(QueryResponses)]
// pub enum AddressQueryMsg {
//     /// Default oracle adapter configuration
//     #[returns(AccountConfig)]
//     Config {},
//     /// Returns the value of given tokens on account
//     #[returns(TokensValueResponse)]
//     TokensValue {
//         address: String,
//         identifiers: Vec<AssetEntry>,
//     },
// }

// TODO: both of them actually address, figure out not confusing name
#[cosmwasm_schema::cw_serde]
pub enum ProxyOrAddr {
    /// Address of the proxy account
    Proxy(String),
    /// Arbitrary address
    Addr(String),
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum OracleQueryMsg {
    /// Oracle adapter configuration
    /// In case proxy address provided - returns config saved by user
    /// Otherwise returns default value, saved by Oracle admin
    #[returns(OracleConfig)]
    Config { proxy_address: Option<String> },
    /// Returns the total value of the assets held by provided account
    #[returns(TokensValueResponse)]
    TotalValue { proxy_address: String },
    /// Returns the value of given tokens by provided account
    #[returns(TokensValueResponse)]
    TokensValue {
        proxy_or_address: ProxyOrAddr,
        identifiers: Vec<AssetEntry>,
    },
    /// Returns the amounts of specified tokens provided account holds
    #[returns(HoldingAmountsResponse)]
    HoldingAmounts {
        proxy_or_address: ProxyOrAddr,
        identifiers: Vec<AssetEntry>,
    },
    /// Returns price sources for given asset entries
    #[returns(AssetPriceSourcesResponse)]
    AssetPriceSources {
        proxy_address: Option<String>,
        identifier: Vec<AssetEntry>,
    },
    /// Returns list of identifiers
    #[returns(AssetIdentifiersResponse)]
    AssetIdentifiers {
        proxy_address: Option<String>,
        start_after: Option<AssetEntry>,
        limit: Option<u8>,
    },
    /// Returns base asset
    #[returns(BaseAssetResponse)]
    BaseAsset { proxy_address: Option<String> },
}

/// Oracle configuration
#[cosmwasm_schema::cw_serde]
pub struct OracleConfig {
    /// Age limit of external quoted price
    pub external_age_max: u64,
}

/// Response from TokensValue
#[cosmwasm_schema::cw_serde]
pub struct TokensValueResponse {
    /// Tokens value relative to base denom
    pub tokens_value: TotalValue,
    /// Tokens value relative to USD quoted by external oracle provider
    pub external_tokens_value: TotalValue,
}

/// Response from HoldingAmounts
#[cosmwasm_schema::cw_serde]
pub struct HoldingAmountsResponse {
    pub amounts: Vec<(AssetEntry, Uint128)>,
}

#[cosmwasm_schema::cw_serde]
pub struct BaseAssetResponse {
    pub base_asset: AssetInfo,
}

/// Human readable config for a single asset
#[cosmwasm_schema::cw_serde]
pub struct AssetPriceSourcesResponse {
    pub sources: Vec<(AssetEntry, UncheckedPriceSource<OraclePriceSource>)>,
}

#[cosmwasm_schema::cw_serde]
pub struct AssetIdentifiersResponse {
    pub identifiers: Vec<AssetEntry>,
}
