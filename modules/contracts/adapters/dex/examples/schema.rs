use std::{env::current_dir, fs::create_dir_all};

use abstract_dex_adapter::{contract::DexAdapter, msg::SimulateSwapResponse};
use cosmwasm_schema::{export_schema_with_title, remove_schemas, schema_for};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    DexAdapter::export_schema(&out_dir);
    export_schema_with_title(
        &schema_for!(SimulateSwapResponse),
        &out_dir,
        "AdapterResponse",
    );
}
