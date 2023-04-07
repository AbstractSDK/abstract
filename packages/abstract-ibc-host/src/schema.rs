use crate::{state::ContractError, Host};
use abstract_sdk::base::{InstantiateEndpoint, MigrateEndpoint, QueryEndpoint};
use cosmwasm_schema::{export_schema_with_title, schema_for};
use schemars::JsonSchema;
use serde::Serialize;
use std::path::Path;

impl<
        Error: ContractError,
        CustomExecMsg: Serialize + JsonSchema,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema,
        CustomMigrateMsg: Serialize + JsonSchema,
        SudoMsg: Serialize + JsonSchema,
        ReceiveMsg: Serialize + JsonSchema,
    >
    Host<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg, ReceiveMsg>
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
