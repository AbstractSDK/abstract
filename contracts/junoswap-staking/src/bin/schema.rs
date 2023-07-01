use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;
use junoswap_staking::msg::{ExecuteMsg, MigrateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: Empty,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    }
}
