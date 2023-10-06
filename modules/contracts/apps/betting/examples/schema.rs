use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use betting_app::contract::BetApp;
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    BetApp::export_schema(&out_dir);
}
