use cosmwasm_schema::write_api;

use gauge_adapter::msg::{AdapterQueryMsg, ExecuteMsg, InstantiateMsg, MigrateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: AdapterQueryMsg,
        migrate: MigrateMsg,
    }
}
