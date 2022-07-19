use abstract_os::{
    manager::{
        ExecuteMsg, InstantiateMsg, ManagerModuleInfo, QueryConfigResponse, QueryInfoResponse,
        QueryModuleAddressesResponse, QueryModuleInfosResponse, QueryModuleVersionsResponse,
        QueryMsg,
    },
    objects::module::Module,
};
use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use abstract_os::manager::state::OsInfo;
use cosmwasm_std::Binary;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    // TODO: failing because of the array, need to delete update_module_addresses
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(OsInfo), &out_dir);
    export_schema(&schema_for!(ManagerModuleInfo), &out_dir);
    export_schema(&schema_for!(Module), &out_dir);
    // TODO:
    export_schema(&schema_for!(Binary), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema_with_title(
        &schema_for!(QueryModuleVersionsResponse),
        &out_dir,
        "ModuleVersionsResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryInfoResponse),
        &out_dir,
        "InfoResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryConfigResponse),
        &out_dir,
        "ConfigResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryModuleInfosResponse),
        &out_dir,
        "ModuleInfosResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryModuleAddressesResponse),
        &out_dir,
        "ModuleAddressesResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryConfigResponse),
        &out_dir,
        "QueryOsConfigResponse",
    );
}
