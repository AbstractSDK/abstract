use cosmwasm_schema::{remove_schemas, write_api};
use my_standalone::msg::{
    MyStandaloneExecuteMsg, MyStandaloneInstantiateMsg, MyStandaloneMigrateMsg,
    MyStandaloneQueryMsg,
};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    #[cfg(feature = "schema")]
    write_api! {
        name: "schema",
        instantiate: MyStandaloneInstantiateMsg,
        query: MyStandaloneQueryMsg,
        execute: MyStandaloneExecuteMsg,
        migrate: MyStandaloneMigrateMsg,
    };
}
