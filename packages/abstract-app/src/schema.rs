use cosmwasm_schema::QueryResponses;
use schemars::JsonSchema;
use serde::Serialize;

use abstract_core::app::{AppExecuteMsg, AppQueryMsg};
use {
    crate::{ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint},
    abstract_sdk::core::app::AppConfigResponse,
    cosmwasm_schema::{export_schema_with_title, schema_for, write_api},
    cw_controllers::AdminResponse,
    std::path::Path,
};

use crate::{AppContract, AppError};

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
        SudoMsg: Serialize + JsonSchema,
    >
    AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
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
