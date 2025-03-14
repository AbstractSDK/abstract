use std::{env::current_dir, fs::create_dir_all};

use abstract_oracle_adapter::contract::OracleAdapter;
use cosmwasm_schema::remove_schemas;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    OracleAdapter::export_schema(&out_dir);
}
