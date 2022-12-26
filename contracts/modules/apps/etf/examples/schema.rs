use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for, write_api};
use cosmwasm_std::Empty;

use abstract_sdk::os::etf::{EtfExecuteMsg, EtfInstantiateMsg, EtfQueryMsg, StateResponse};
use etf::contract::EtfApp;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    // This is temporary until we can use the new cosmwasm-schema
    write_api! {
        name: "schema",
        instantiate: EtfInstantiateMsg,
        query: EtfQueryMsg,
        execute: EtfExecuteMsg,
        migrate: Empty,
    };

    EtfApp::export_schema(&out_dir);
    export_schema(&schema_for!(StateResponse), &out_dir);
}
