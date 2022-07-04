use cw_asset::{AssetInfo, AssetInfoUnchecked};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod state {
    use cosmwasm_std::Addr;
    use cw_asset::AssetInfo;
    use cw_controllers::Admin;
    use cw_storage_plus::Map;

    pub const PAIR_POSTFIX: &str = "pair";

    pub const ADMIN: Admin = Admin::new("admin");
    // stores name and address of tokens and pairs
    // LP token key: "ust_luna"
    pub const ASSET_ADDRESSES: Map<&str, AssetInfo> = Map::new("assets");

    // Pair key: "ust_luna_pair"
    pub const CONTRACT_ADDRESSES: Map<&str, Addr> = Map::new("contracts");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Updates the addressbook
    UpdateContractAddresses {
        to_add: Vec<(String, String)>,
        to_remove: Vec<String>,
    },
    UpdateAssetAddresses {
        to_add: Vec<(String, AssetInfoUnchecked)>,
        to_remove: Vec<String>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
