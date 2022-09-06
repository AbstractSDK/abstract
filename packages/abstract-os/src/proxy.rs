//! # OS Proxy
//!
//! `abstract_os::proxy` hold all the assets associated with the OS instance. It accepts Cosmos messages from whitelisted addresses and executes them.
//!
//! ## Description
//! The proxy is part of the Core OS contracts along with the [`crate::manager`] contract.
//! This contract is responsible for executing Cosmos messages and calculating the value of its internal assets.
//!
//! ## Proxy assets
//! [Proxy assets](crate::objects::proxy_asset) are what allow the proxy contract to provide value queries for its assets. It needs to be configured using the [`ExecuteMsg::UpdateAssets`] endpoint.
//! After configurating the proxy assets [`QueryMsg::TotalValue`] can be called to get the total holding value.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CosmosMsg, Empty, Uint128};

use crate::objects::{
    proxy_asset::{ProxyAsset, UncheckedProxyAsset},
    AssetEntry,
};

pub mod state {
    pub use crate::objects::core::OS_ID;
    use cw_controllers::Admin;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use cosmwasm_std::Addr;
    use cw_storage_plus::{Item, Map};

    use crate::objects::{asset_entry::AssetEntry, memory::Memory, proxy_asset::ProxyAsset};
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
    pub struct State {
        pub modules: Vec<Addr>,
    }
    pub const MEMORY: Item<Memory> = Item::new("\u{0}{6}memory");
    pub const STATE: Item<State> = Item::new("\u{0}{5}state");
    pub const ADMIN: Admin = Admin::new("admin");
    pub const VAULT_ASSETS: Map<AssetEntry, ProxyAsset> = Map::new("proxy_assets");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub os_id: u32,
    pub memory_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Sets the admin
    SetAdmin { admin: String },
    /// Executes the provided messages if sender is whitelisted
    ModuleAction { msgs: Vec<CosmosMsg<Empty>> },
    /// Adds the provided address to whitelisted dapps
    AddModule { module: String },
    /// Removes the provided address from the whitelisted dapps
    RemoveModule { module: String },
    /// Updates the VAULT_ASSETS map
    UpdateAssets {
        to_add: Vec<UncheckedProxyAsset>,
        to_remove: Vec<String>,
    },
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns [`QueryConfigResponse`]
    Config {},
    /// Returns the total value of all held assets
    /// [`QueryTotalValueResponse`]
    TotalValue {},
    /// Returns the value of one specific asset
    /// [`QueryHoldingValueResponse`]
    HoldingValue { identifier: String },
    /// Returns the amount of specified tokens this contract holds
    /// [`QueryHoldingAmountResponse`]
    HoldingAmount { identifier: String },
    /// Returns the VAULT_ASSETS value for the specified key
    /// [`QueryAssetConfigResponse`]
    AssetConfig { identifier: String },
    /// Returns [`QueryAssetsResponse`]
    Assets {
        page_token: Option<String>,
        page_size: Option<u8>,
    },
    /// Returns [`QueryAssetsResponse`]
    CheckValidity {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryConfigResponse {
    pub modules: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryTotalValueResponse {
    pub value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryHoldingValueResponse {
    pub value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryValidityResponse {
    /// Assets that have unresolvable dependencies in their value calculation
    pub unresolvable_assets: Option<Vec<AssetEntry>>,
    /// Assets that are missing in the VAULT_ASSET map which caused some assets to be unresolvable.
    pub missing_dependencies: Option<Vec<AssetEntry>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryHoldingAmountResponse {
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryAssetConfigResponse {
    pub proxy_asset: ProxyAsset,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryAssetsResponse {
    pub assets: Vec<(AssetEntry, ProxyAsset)>,
}

/// Query message to external contract to get asset value
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ValueQueryMsg {
    pub asset: AssetEntry,
    pub amount: Uint128,
}
/// External contract value response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ExternalValueResponse {
    pub value: Uint128,
}
