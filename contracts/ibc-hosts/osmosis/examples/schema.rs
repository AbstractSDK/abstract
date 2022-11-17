use std::env::current_dir;
use std::fs::create_dir_all;

use abstract_sdk::os::{
    dex::{DexRequestMsg, SimulateSwapResponse},
    extension::{ExecuteMsg, ExtensionConfigResponse, TradersResponse},
};
use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use osmosis_host::contract::OsmoHost;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    OsmoHost::export_schema(&out_dir);

    export_schema_with_title(
        &schema_for!(SimulateSwapResponse),
        &out_dir,
        "ExtensionResponse",
    );

    // export_schema_with_title(&schema_for!(ExecuteMsg<DexRequestMsg>), &out_dir, "ExecuteMsg");
    export_schema_with_title(&schema_for!(TradersResponse), &out_dir, "TradersResponse");
    export_schema_with_title(
        &schema_for!(ExtensionConfigResponse),
        &out_dir,
        "ConfigResponse",
    );

    // export_schema_with_title(&schema_for!(ExtensionQueryMsg), &out_dir, "QueryMsg");
}
