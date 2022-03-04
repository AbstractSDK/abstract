use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use dapp_template::msg::{ExecuteMsg, QueryMsg};
use pandora_os::core::treasury::dapp_base::msg::BaseInstantiateMsg;
use pandora_os::core::treasury::dapp_base::state::BaseState;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(BaseInstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(BaseState), &out_dir);
}
