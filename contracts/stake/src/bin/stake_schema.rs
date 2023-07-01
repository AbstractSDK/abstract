use cosmwasm_schema::write_api;
use wyndex::stake::InstantiateMsg;
use wyndex_stake::msg::{ExecuteMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
