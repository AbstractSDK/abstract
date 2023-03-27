use abstract_sdk::core::version_control::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
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
    // export_schema(&schema_for!(ExecuteMsg), &out_dir);
    // export_schema(&schema_for!(QueryMsg), &out_dir);
    // export_schema(&schema_for!(ModuleInfo), &out_dir);
    // // export_schema(&schema_for!(EnabledModulesResponse), &out_dir);
    // export_schema_with_title(&schema_for!(CodeIdResponse), &out_dir, "CodeIdResponse");
    // export_schema_with_title(&schema_for!(ConfigResponse), &out_dir, "ConfigResponse");
    // export_schema_with_title(
    //     &schema_for!(ApiAddressesResponse),
    //     &out_dir,
    //     "ApiAddressesResponse",
    // );
    // export_schema_with_title(
    //     &schema_for!(ApiAddressResponse),
    //     &out_dir,
    //     "ApiAddressResponse",
    // );
    // export_schema_with_title(&schema_for!(CodeIdResponse), &out_dir, "CodeIdResponse");
    // export_schema_with_title(&schema_for!(CodeIdsResponse), &out_dir, "CodeIdsResponse");
    // export_schema_with_title(&schema_for!(AccountBaseResponse), &out_dir, "AccountBaseResponse");
}
