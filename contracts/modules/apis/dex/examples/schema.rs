use std::env::current_dir;
use std::fs::create_dir_all;

use abstract_api::ApiResult;
use abstract_os::{dex::{QueryMsg, RequestMsg, SimulateSwapResponse}, api::{ExecuteMsg, QueryTradersResponse, QueryApiConfigResponse, ApiQueryMsg}};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for, export_schema_with_title};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema_with_title(&schema_for!(ApiQueryMsg<QueryMsg>), &out_dir, "QueryMsg");

    export_schema_with_title(&schema_for!(SimulateSwapResponse), &out_dir, "ApiResponse");


    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema_with_title(&schema_for!(QueryTradersResponse), &out_dir, "TradersResponse");
    export_schema_with_title(&schema_for!(QueryApiConfigResponse), &out_dir, "ConfigResponse");

    // export_schema_with_title(&schema_for!(ApiQueryMsg), &out_dir, "QueryMsg");
    export_schema_with_title(&schema_for!(ExecuteMsg<RequestMsg>), &out_dir, "ExecuteMsg");
}
