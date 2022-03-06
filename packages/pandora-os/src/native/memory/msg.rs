use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use terra_rust_script_derive::contract;
use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, contract)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, contract)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Updates the addressbook
    UpdateContractAddresses {
        to_add: Vec<(String, String)>,
        to_remove: Vec<String>,
    },
    UpdateAssetAddresses {
        to_add: Vec<(String, AssetInfo)>,
        to_remove: Vec<String>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, contract)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Queries assets based on name
    QueryAssets {
        names: Vec<String>,
    },
    QueryContracts {
        names: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetQueryResponse {
    pub assets: Vec<(String, AssetInfo)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractQueryResponse {
    pub contracts: Vec<(String, String)>,
}
