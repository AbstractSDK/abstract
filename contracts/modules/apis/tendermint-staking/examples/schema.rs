use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for, export_schema_with_title};

use abstract_os::{tendermint_staking::{QueryMsg, RequestMsg}, add_on::AddOnQueryMsg, api::ExecuteMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(RequestMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema_with_title(&schema_for!(AddOnQueryMsg), &out_dir, "BaseResponse");
    export_schema_with_title(&schema_for!(ExecuteMsg<RequestMsg>), &out_dir, "ExecuteMsg");
}
