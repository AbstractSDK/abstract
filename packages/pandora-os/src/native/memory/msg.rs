use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
 use terra_rust_script_derive::CosmWasmContract;
use terraswap::asset::AssetInfo;
use cw_asset::AssetInfoBase;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Updates the addressbook
    UpdateContractAddresses {
        to_add: Vec<(String, String)>,
        to_remove: Vec<String>,
    },
    UpdateAssetAddresses {
        to_add: Vec<(String, AssetInfoBase<Addr>)>,
        to_remove: Vec<String>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
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
    pub assets: Vec<(String, AssetInfoBase<Addr>)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractQueryResponse {
    pub contracts: Vec<(String, String)>,
}
