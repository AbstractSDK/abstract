use abstract_core::app;
use cosmwasm_schema::{cw_serde, write_api, QueryResponses};
use cosmwasm_std::Empty;

#[cw_serde]
#[derive(QueryResponses)]
pub enum EmptyQuery {}

fn main() {
    write_api! {
        name: "app-schema",
        instantiate: app::InstantiateMsg<Empty>,
        query: app::QueryMsg<EmptyQuery>,
        execute: app::ExecuteMsg,
        migrate: app::MigrateMsg,
    };
}
