use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use abstract_os::{
    version_control::{
        CodeIdResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, OsCoreResponse, QueryMsg,
    },
    {manager::EnabledModulesResponse, modules::ModuleInfo},
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(ModuleInfo), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    // export_schema(&schema_for!(EnabledModulesResponse), &out_dir);
    export_schema_with_title(
        &schema_for!(EnabledModulesResponse),
        &out_dir,
        "QueryEnabledModulesResponse",
    );
    export_schema_with_title(
        &schema_for!(CodeIdResponse),
        &out_dir,
        "QueryCodeIdResponse",
    );
    export_schema_with_title(
        &schema_for!(OsCoreResponse),
        &out_dir,
        "QueryOsAddrResponse",
    );
}
