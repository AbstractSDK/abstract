use abstract_os::{
    manager::{
        ConfigQueryResponse, EnabledModulesResponse, ExecuteMsg, InstantiateMsg,
        ModuleQueryResponse, QueryMsg, VersionsQueryResponse,
    },
    modules::Module,
};
use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
use std::{env::current_dir, fs::create_dir_all};

use abstract_os::manager::state::Config;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Empty};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    // TODO: failing because of the array, need to delete update_module_addresses
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(Module), &out_dir);
    // TODO:
    export_schema(&schema_for!(Binary), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema_with_title(
        &schema_for!(VersionsQueryResponse),
        &out_dir,
        "QueryVersionsResponse",
    );
    export_schema_with_title(
        &schema_for!(ModuleQueryResponse),
        &out_dir,
        "QueryModulesResponse",
    );
    export_schema_with_title(
        &schema_for!(EnabledModulesResponse),
        &out_dir,
        "QueryEnabledModulesResponse",
    );
    export_schema_with_title(
        &schema_for!(ConfigQueryResponse),
        &out_dir,
        "QueryOsConfigResponse",
    );
}
