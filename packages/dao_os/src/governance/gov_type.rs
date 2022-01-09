use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, Env, QueryRequest, StdError, StdResult, Uint128, WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceDetails {
    Monarchy {
        owner: String,
    },
    MultiSignature {
        total_members: u8,
        threshold_votes: u8,
        members: Vec<String>,
    },
    TokenWeighted {
        token_addr: String,
    },
}
