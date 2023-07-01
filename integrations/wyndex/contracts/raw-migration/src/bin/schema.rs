use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;
use raw_migration::msg::{ExecuteMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: Empty,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
