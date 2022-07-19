use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use abstract_os::{
    objects::module::ModuleInfo,
    version_control::{
        ExecuteMsg, InstantiateMsg, QueryApiAddressResponse, QueryApiAddressesResponse,
        QueryCodeIdResponse, QueryCodeIdsResponse, QueryConfigResponse, QueryMsg,
        QueryOsCoreResponse,
    },
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
    // export_schema(&schema_for!(EnabledModulesResponse), &out_dir);
    export_schema_with_title(
        &schema_for!(QueryCodeIdResponse),
        &out_dir,
        "CodeIdResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryConfigResponse),
        &out_dir,
        "ConfigResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryApiAddressesResponse),
        &out_dir,
        "ApiAddressesResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryApiAddressResponse),
        &out_dir,
        "ApiAddressResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryCodeIdResponse),
        &out_dir,
        "CodeIdResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryCodeIdsResponse),
        &out_dir,
        "CodeIdsResponse",
    );
    export_schema_with_title(
        &schema_for!(QueryOsCoreResponse),
        &out_dir,
        "OsCoreResponse",
    );
}
