use abstract_core::adapter;
use cosmwasm_schema::write_api;

mod query {
    #[cosmwasm_schema::cw_serde]
    #[derive(cosmwasm_schema::QueryResponses)]
    pub enum Empty {}
}

mod execute {
    #[cosmwasm_schema::cw_serde]
    pub struct Empty {}
}

fn main() {
    write_api! {
        name: "adapter-schema",
        instantiate: adapter::InstantiateMsg<execute::Empty>,
        query: adapter::QueryMsg<query::Empty>,
        execute: adapter::ExecuteMsg,
        migrate: execute::Empty,
    };
}
