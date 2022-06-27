use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, CosmosMsg, Empty, StdResult, Uint128, WasmMsg};
use terra_rust_script_derive::CosmWasmContract;

use crate::core::proxy::proxy_assets::ProxyAsset;
use cw_asset::AssetInfo;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub os_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, CosmWasmContract)]
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
        to_add: Vec<ProxyAsset>,
        to_remove: Vec<AssetInfo>,
    },
}

/// MigrateMsg allows a privileged contract administrator to run
/// a migration on the contract. In this case it is just migrating
/// from one terra code to the same code, but taking advantage of the
/// migration step to set a new validator.
///
/// Note that the contract doesn't enforce permissions here, this is done
/// by blockchain logic (in the future by blockchain governance)
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
/// Constructs the proxy dapp action message used by all dApps.
pub fn send_to_proxy(msgs: Vec<CosmosMsg>, proxy_address: &Addr) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address.to_string(),
        msg: to_binary(&ExecuteMsg::ModuleAction { msgs })?,
        funds: vec![],
    }))
}
