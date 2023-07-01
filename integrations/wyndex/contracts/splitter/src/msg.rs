use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Decimal;

#[cw_serde]
pub struct InstantiateMsg {
    // List of addresses and their weights.
    // Weights must sum up to 1.0
    pub addresses: Vec<(String, Decimal)>,
    // List of cw20 token addresses to check for balance
    pub cw20_contracts: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Transfers tokens send to this contract based on weights from configuration.
    // Any user can execute it
    SendTokens {
        // Provide denoms of native tokens to check
        native_denoms: Vec<String>,
        // Provide addresses of cw20 contracts to check
        // If None, contract will query adresses from Config
        cw20_addresses: Option<Vec<String>>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::Config)]
    Config {},
}

/// Remove old configuration and replace it with new one
#[cw_serde]
pub struct MigrateMsg {
    pub new_addresses: Vec<(String, Decimal)>,
    pub new_cw20_contracts: Vec<String>,
}
