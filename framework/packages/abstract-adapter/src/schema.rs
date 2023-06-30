use std::path::Path;

use cosmwasm_schema::{export_schema_with_title, schema_for, write_api, QueryResponses};
use cosmwasm_std::Empty;
use schemars::JsonSchema;
use serde::Serialize;

use abstract_core::adapter::{AdapterExecuteMsg, AdapterQueryMsg};
use abstract_sdk::{
    base::{ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint},
    core::adapter::{AdapterConfigResponse, AuthorizedAddressesResponse},
};

use crate::{AdapterContract, AdapterError};

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AdapterError>
            + From<abstract_sdk::AbstractSdkError>
            + 'static,
        CustomExecMsg: Serialize + JsonSchema + AdapterExecuteMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema + AdapterQueryMsg + QueryResponses,
        ReceiveMsg: Serialize + JsonSchema,
        SudoMsg,
    > AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
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
        export_schema_with_title(
            &schema_for!(AdapterConfigResponse),
            out_dir,
            "ConfigResponse",
        );
    }
}
