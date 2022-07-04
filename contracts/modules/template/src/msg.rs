use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use abstract_os::pandora_dapp::DappExecuteMsg;
use abstract_os::pandora_dapp::DappQueryMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(DappExecuteMsg),
    // Add dapp-specific messages here
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(DappQueryMsg),
    // Add dapp-specific queries here
}
