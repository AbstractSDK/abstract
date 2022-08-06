use std::env::current_dir;
use std::fs::create_dir_all;

use abstract_os::memory::{
    QueryAssetListResponse, QueryAssetsResponse, QueryContractListResponse, QueryContractsResponse,
};
use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use abstract_os::memory::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    // export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema_with_title(&schema_for!(QueryContractsResponse), &out_dir, "ContractsResponse");
    export_schema_with_title(&schema_for!(QueryAssetsResponse), &out_dir, "AssetsResponse");
    export_schema_with_title(&schema_for!(QueryContractListResponse), &out_dir, "ContractListResponse");
    export_schema_with_title(&schema_for!(QueryAssetListResponse), &out_dir, "AssetListResponse");
}
