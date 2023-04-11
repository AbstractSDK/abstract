use crate::{ApiContract, ApiError};
use abstract_core::api::{ApiExecuteMsg, ApiQueryMsg};
use abstract_sdk::{
    base::{ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint},
    core::api::{ApiConfigResponse, AuthorizedAddressesResponse},
};
use cosmwasm_schema::{export_schema_with_title, schema_for, write_api, QueryResponses};
use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::Serialize;
use std::path::Path;

impl<
        Error: From<cosmwasm_std::StdError>
            + From<ApiError>
            + From<abstract_sdk::AbstractSdkError>
            + 'static,
        CustomExecMsg: Serialize + JsonSchema + ApiExecuteMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema + ApiQueryMsg + QueryResponses,
        ReceiveMsg: Serialize + JsonSchema,
        SudoMsg,
    > ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    pub fn export_schema(out_dir: &Path) {
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
        export_schema_with_title(
            &schema_for!(AuthorizedAddressesResponse),
            out_dir,
            "AuthorizedAddressesResponse",
        );
        export_schema_with_title(&schema_for!(ApiConfigResponse), out_dir, "ConfigResponse");
    }
}
