use crate::{
    AppContract, AppError, ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint,
};
use abstract_os::app::{AppExecuteMsg, AppQueryMsg};
use abstract_sdk::os::app::AppConfigResponse;
use cosmwasm_schema::{export_schema_with_title, schema_for, write_api, QueryResponses};
use cw_controllers::AdminResponse;
use schemars::JsonSchema;
use serde::Serialize;
use std::path::Path;

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AppError>
            + From<abstract_sdk::AbstractSdkError>
            + 'static,
        CustomExecMsg: Serialize + JsonSchema + AppExecuteMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema + AppQueryMsg + QueryResponses,
        CustomMigrateMsg: Serialize + JsonSchema,
        ReceiveMsg: Serialize + JsonSchema,
    >
    AppContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    pub fn export_schema(out_dir: &Path) {
        // write out the module-specific schema
        write_api! {
            name: "module-schema",
            instantiate: CustomInitMsg,
            query: CustomQueryMsg,
            execute: CustomExecMsg,
            migrate: CustomMigrateMsg,
        };

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
