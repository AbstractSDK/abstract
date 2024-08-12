use std::path::Path;

use abstract_sdk::base::{ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint};
use abstract_std::{
    adapter,
    adapter::{AdapterExecuteMsg, AdapterQueryMsg},
};
use cosmwasm_schema::{export_schema_with_title, schema_for, write_api, QueryResponses};
use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{AdapterContract, AdapterError};

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AdapterError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_std::AbstractError>
            + 'static,
        CustomExecMsg: Serialize + JsonSchema + AdapterExecuteMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema + AdapterQueryMsg + QueryResponses,
        SudoMsg,
    > AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    pub fn export_schema(out_dir: &Path) {
        write_api! {
            name: "schema",
            instantiate: adapter::InstantiateMsg<CustomInitMsg>,
            query: adapter::QueryMsg<CustomQueryMsg>,
            execute: adapter::ExecuteMsg<CustomExecMsg>,
            migrate: Empty,
        };

        // write out the module schema
        write_api! {
            name: "module-schema",
            instantiate: CustomInitMsg,
            query: CustomQueryMsg,
            execute: CustomExecMsg,
            migrate: Empty,
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
    }
}
