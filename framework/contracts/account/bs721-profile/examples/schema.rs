
use bs721_profile::*;
use bs_profile::Metadata;
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg<Metadata>,
        migrate: MigrateMsg,
    };
}
