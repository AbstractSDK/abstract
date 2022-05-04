use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DappStateResponse {
    pub proxy_address: Addr,
    pub memory_address: Addr,
    pub traders: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TradersResponse {
    /// Contains all traders in lexicographical ordering
    /// TODO: If there are more than `limit`, use `start_from` in future queries
    /// to achieve pagination.
    pub traders: Vec<Addr>,
}
