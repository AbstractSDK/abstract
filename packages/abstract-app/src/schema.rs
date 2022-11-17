use std::path::Path;

use crate::{
    AppContract, AppError, ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint,
};
use abstract_sdk::os::app::AppConfigResponse;
use cosmwasm_schema::{export_schema_with_title, schema_for};
use cw_controllers::AdminResponse;
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        CustomExecMsg: Serialize + JsonSchema,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema,
        CustomMigrateMsg: Serialize + JsonSchema,
        ReceiveMsg: Serialize + JsonSchema,
    >
    AppContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    pub fn export_schema(out_dir: &Path) {
        export_schema_with_title(
            &schema_for!(<Self as ExecuteEndpoint>::ExecuteMsg),
            out_dir,
            "ExecuteMsg",
        );
        export_schema_with_title(
            &schema_for!(<Self as InstantiateEndpoint>::InstantiateMsg),
            out_dir,
            "InstantiateMsg",
        );
        export_schema_with_title(
            &schema_for!(<Self as QueryEndpoint>::QueryMsg),
            out_dir,
            "QueryMsg",
        );
        export_schema_with_title(
            &schema_for!(<Self as MigrateEndpoint>::MigrateMsg),
            out_dir,
            "MigrateMsg",
        );
        export_schema_with_title(&schema_for!(AdminResponse), out_dir, "AdminResponse");
        export_schema_with_title(&schema_for!(AppConfigResponse), out_dir, "ConfigResponse");
    }
}
