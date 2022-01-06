use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, CosmosMsg, Empty, StdResult, Uint128, WasmMsg};

use crate::treasury::vault_assets::VaultAsset;
use terraswap::asset::AssetInfo;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Sets the admin
    SetAdmin { admin: String },
    /// Executes the provided messages if sender is whitelisted
    DAppAction { msgs: Vec<CosmosMsg<Empty>> },
    /// Adds the provided address to whitelisted dapps
    AddDApp { dapp: String },
    /// Removes the provided address from the whitelisted dapps
    RemoveDApp { dapp: String },
    /// Updates the VAULT_ASSETS map
    UpdateAssets {
        to_add: Vec<VaultAsset>,
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
    /// Returns the treasury Config
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
    pub dapps: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalValueResponse {
    pub value: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HoldingValueResponse {
    pub value: Uint128,
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
/// Constructs the treasury dapp action message used by all dApps.
pub fn send_to_treasury(
    msgs: Vec<CosmosMsg>,
    treasury_address: &Addr,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: treasury_address.to_string(),
        msg: to_binary(&ExecuteMsg::DAppAction { msgs })?,
        funds: vec![],
    }))
}
