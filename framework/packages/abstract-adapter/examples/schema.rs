use abstract_core::adapter;
use cosmwasm_schema::{cw_serde, write_api, QueryResponses};
use cosmwasm_std::Empty;

#[cw_serde]
#[derive(QueryResponses)]
pub enum EmptyQuery {}

fn main() {
    write_api! {
        name: "adapter-schema",
        instantiate: adapter::InstantiateMsg<Empty>,
        query: adapter::QueryMsg<EmptyQuery>,
        execute: adapter::ExecuteMsg,
        migrate: Empty,
    };
}
