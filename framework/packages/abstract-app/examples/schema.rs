use std::{env::current_dir, fs::create_dir_all};

use abstract_core::app;
use cosmwasm_schema::{remove_schemas, write_api};

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
        name: "app-schema",
        instantiate: app::InstantiateMsg<execute::Empty>,
        query: app::QueryMsg<query::Empty>,
        execute: app::ExecuteMsg,
        migrate: app::MigrateMsg,
    };
}
