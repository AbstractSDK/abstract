use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, CosmosMsg, Decimal, Empty, StdResult, Uint128, WasmMsg};
use cw_asset::{AssetInfo, AssetInfoUnchecked, AssetUnchecked};

use crate::objects::proxy_assets::ProxyAsset;

pub mod state {
    pub use crate::objects::core::OS_ID;
    use cw_controllers::Admin;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use cosmwasm_std::Addr;
    use cw_storage_plus::{Item, Map};

    use crate::objects::proxy_assets::ProxyAsset;
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct State {
        pub modules: Vec<Addr>,
    }
    pub const STATE: Item<State> = Item::new("\u{0}{5}state");
    pub const ADMIN: Admin = Admin::new("admin");
    pub const VAULT_ASSETS: Map<&str, ProxyAsset> = Map::new("proxy_assets");
}

/// Constructs the proxy dapp action message used by all dApps.
pub fn send_to_proxy(msgs: Vec<CosmosMsg>, proxy_address: &Addr) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        msg: to_binary(&ExecuteMsg::ModuleAction { msgs })?,
        funds: vec![],
    }))
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UncheckedProxyAsset {
    pub asset: AssetUnchecked,
    // The value reference provides the tooling to get the value of the holding
    // relative to the base asset.
    pub value_reference: Option<UncheckedValueRef>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UncheckedValueRef {
    /// A pool address of an asset/asset pair
    /// Both assets must be defined in the Vault_assets state
    Pool {
        pair_address: String,
    },
    // Liquidity pool addr for LP tokens
    Liquidity {
        pool_address: String,
    },
    // Or a Proxy, the proxy also takes a Decimal (the multiplier)
    // Asset will be valued as if they are Proxy tokens
    Proxy {
        proxy_asset: AssetInfoUnchecked,
        multiplier: Decimal,
    },
    // Query an external contract to get the value
    External {
        contract_address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub os_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
        to_remove: Vec<AssetInfo>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the proxy Config
    Config {},
    /// Returns the total value of all held assets
    TotalValue {},
    /// Returns the value of one specific asset
    HoldingValue { identifier: String },
    /// Returns the amount of specified tokens this contract holds
    HoldingAmount { identifier: String },
    /// Returns the VAULT_ASSETS value for the specified key
    VaultAssetConfig { identifier: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub modules: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalValueResponse {
    pub value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HoldingValueResponse {
    pub value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HoldingAmountResponse {
    pub value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultAssetConfigResponse {
    pub value: ProxyAsset,
}

/// Query message to external contract to get asset value
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ValueQueryMsg {
    pub asset_info: AssetInfo,
    pub amount: Uint128,
}
/// External contract value response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExternalValueResponse {
    pub value: Uint128,
}
