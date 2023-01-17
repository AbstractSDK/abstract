use crate::{Host, HostError};
use abstract_sdk::base::endpoints::{InstantiateEndpoint, MigrateEndpoint, QueryEndpoint};
use cosmwasm_schema::{export_schema_with_title, schema_for};
use schemars::JsonSchema;
use serde::Serialize;
use std::path::Path;

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg: Serialize + JsonSchema,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema,
        CustomMigrateMsg: Serialize + JsonSchema,
        ReceiveMsg: Serialize + JsonSchema,
    > Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    pub fn export_schema(out_dir: &Path) {
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
