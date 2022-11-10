use std::path::Path;

use crate::{ApiContract, ApiError};
use abstract_os::api::{ApiConfigResponse, TradersResponse};
use abstract_sdk::{ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint};
use cosmwasm_schema::{export_schema_with_title, schema_for};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg: Serialize + JsonSchema,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema,
        ReceiveMsg: Serialize + JsonSchema,
    > ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
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
        export_schema_with_title(&schema_for!(TradersResponse), out_dir, "TradersResponse");
        export_schema_with_title(&schema_for!(ApiConfigResponse), out_dir, "ConfigResponse");
    }
}
