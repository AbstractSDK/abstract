use cosmwasm_schema::QueryResponses;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use abstract_core::app::{self, AppExecuteMsg, AppQueryMsg};
use {
    crate::{ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint},
    cosmwasm_schema::{export_schema_with_title, schema_for, write_api},
    std::path::Path,
};

use crate::{AppContract, AppError};

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AppError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_core::AbstractError>
            + 'static,
        CustomExecMsg: Serialize + JsonSchema + AppExecuteMsg,
        CustomInitMsg: Serialize + DeserializeOwned + JsonSchema,
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
        write_api! {
            name: "schema",
            instantiate: app::InstantiateMsg<CustomInitMsg>,
            query: app::QueryMsg<CustomQueryMsg>,
            execute: app::ExecuteMsg<CustomExecMsg, ReceiveMsg>,
            migrate: app::MigrateMsg<CustomMigrateMsg>,
        };

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
    }
}
