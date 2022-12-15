use std::path::Path;

use crate::{ExtensionContract, ExtensionError};
use abstract_os::extension::{ExtensionExecuteMsg, ExtensionQueryMsg};
use abstract_sdk::{
    base::endpoints::{ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint},
    os::extension::{ExtensionConfigResponse, TradersResponse},
};
use cosmwasm_schema::{export_schema_with_title, schema_for};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        CustomExecMsg: Serialize + JsonSchema + ExtensionExecuteMsg,
        CustomInitMsg: Serialize + JsonSchema,
        CustomQueryMsg: Serialize + JsonSchema + ExtensionQueryMsg,
        ReceiveMsg: Serialize + JsonSchema,
    > ExtensionContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
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
        export_schema_with_title(
            &schema_for!(ExtensionConfigResponse),
            out_dir,
            "ConfigResponse",
        );
    }
}
