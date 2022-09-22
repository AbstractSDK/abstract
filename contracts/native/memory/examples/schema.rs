use abstract_os::memory::MigrateMsg;
use cosmwasm_schema::write_api;

use abstract_os::memory::{ExecuteMsg, InstantiateMsg, QueryMsg};

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
    // export_schema_with_title(
    //     &schema_for!(ContractsResponse),
    //     &out_dir,
    //     "ContractsResponse",
    // );
    // export_schema_with_title(&schema_for!(AssetsResponse), &out_dir, "AssetsResponse");
    // export_schema_with_title(
    //     &schema_for!(ContractListResponse),
    //     &out_dir,
    //     "ContractListResponse",
    // );
    // export_schema_with_title(
    //     &schema_for!(AssetListResponse),
    //     &out_dir,
    //     "AssetListResponse",
    // );
}
