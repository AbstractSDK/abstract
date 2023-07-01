use cosmwasm_schema::write_api;

use lp_converter::msg::{InstantiateMsg, QueryMsg};
use wyndex::lp_converter::ExecuteMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ExecuteMsg,
        query: QueryMsg,
    }
}
