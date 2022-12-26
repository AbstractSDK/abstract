use std::env::current_dir;
use std::fs::create_dir_all;

use abstract_sdk::os::dex::{DexExecuteMsg, DexQueryMsg, SimulateSwapResponse};
use cosmwasm_schema::{export_schema_with_title, remove_schemas, schema_for, write_api};
use cosmwasm_std::Empty;
use dex::contract::DexApi;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    // Write a modified entry point schema for the Dex API
    write_api! {
        name: "schema",
        query: DexQueryMsg,
        execute: DexExecuteMsg,
        instantiate: Empty,
        migrate: Empty,
    };

    DexApi::export_schema(&out_dir);
    export_schema_with_title(&schema_for!(SimulateSwapResponse), &out_dir, "ApiResponse");
}
