use abstract_sdk::os::manager::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_schema::write_api;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
        migrate: MigrateMsg,
    };
    // let mut out_dir = current_dir().unwrap();
    // out_dir.push("schema");
    // create_dir_all(&out_dir).unwrap();
    // remove_schemas(&out_dir).unwrap();

    // export_schema(&schema_for!(InstantiateMsg), &out_dir);
    // // TODO: failing because of the array, need to delete update_module_addresses
    // export_schema(&schema_for!(ExecuteMsg), &out_dir);
    // export_schema(&schema_for!(OsInfo), &out_dir);
    // export_schema(&schema_for!(ManagerModuleInfo), &out_dir);
    // export_schema(&schema_for!(Module), &out_dir);
    // // TODO:
    // export_schema(&schema_for!(Binary), &out_dir);
    // export_schema(&schema_for!(QueryMsg), &out_dir);
    // export_schema_with_title(
    //     &schema_for!(ModuleVersionsResponse),
    //     &out_dir,
    //     "ModuleVersionsResponse",
    // );
    // export_schema_with_title(&schema_for!(InfoResponse), &out_dir, "InfoResponse");
    // export_schema_with_title(&schema_for!(ConfigResponse), &out_dir, "ConfigResponse");
    // export_schema_with_title(
    //     &schema_for!(ModuleInfosResponse),
    //     &out_dir,
    //     "ModuleInfosResponse",
    // );
    // export_schema_with_title(
    //     &schema_for!(ModuleAddressesResponse),
    //     &out_dir,
    //     "ModuleAddressesResponse",
    // );
    // export_schema_with_title(&schema_for!(ConfigResponse), &out_dir, "OsConfigResponse");
}
