use std::path::Path;

use abstract_core::adapter;
use cosmwasm_schema::{export_schema_with_title, schema_for, write_api, QueryResponses};
use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::Serialize;

use abstract_core::adapter::{AdapterExecuteMsg, AdapterQueryMsg};
use abstract_sdk::base::{ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint};

use crate::{AdapterContract, AdapterError};

impl<
        'a,
        Error: From<cosmwasm_std::StdError>
            + From<AdapterError>
            + From<abstract_sdk::AbstractSdkError>
            + 'static,
        CustomExecMsg: Serialize + JsonSchema + AdapterExecuteMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema + AdapterQueryMsg + QueryResponses,
        ReceiveMsg: Serialize + JsonSchema,
        SudoMsg,
    >
    AdapterContract<'a, Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    pub fn export_schema(out_dir: &Path) {
        write_api! {
            name: "schema",
            instantiate: adapter::InstantiateMsg<CustomInitMsg>,
            query: adapter::QueryMsg<CustomQueryMsg>,
            execute: adapter::ExecuteMsg<CustomExecMsg, ReceiveMsg>,
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
            &schema_for!(
                <AdapterContract<
                    'a,
                    Error,
                    CustomInitMsg,
                    CustomExecMsg,
                    CustomQueryMsg,
                    ReceiveMsg,
                    SudoMsg,
                > as ExecuteEndpoint>::ExecuteMsg
            ),
            out_dir,
            "ExecuteMsg",
        );
        export_schema_with_title(
            &schema_for!(
                <AdapterContract<
                    'a,
                    Error,
                    CustomInitMsg,
                    CustomExecMsg,
                    CustomQueryMsg,
                    ReceiveMsg,
                    SudoMsg,
                > as InstantiateEndpoint>::InstantiateMsg
            ),
            out_dir,
            "InstantiateMsg",
        );
        export_schema_with_title(
            &schema_for!(
                <AdapterContract<
                    'a,
                    Error,
                    CustomInitMsg,
                    CustomExecMsg,
                    CustomQueryMsg,
                    ReceiveMsg,
                    SudoMsg,
                > as QueryEndpoint>::QueryMsg
            ),
            out_dir,
            "QueryMsg",
        );
    }
}
