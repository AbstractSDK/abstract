use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

use crate::vault_info::VaultInfoRaw;
use white_whale::vault_asset::VaultAsset;

pub static LUNA_DENOM: &str = "uluna";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
    pub traders: Vec<CanonicalAddr>,
}

// Allowances
pub const STATE: Item<State> = Item::new("\u{0}{5}state");

// Vault information
pub const VAULT_INFO: Item<VaultInfoRaw> = Item::new("\u{0}{5}vault");

pub const VAULT_ASSETS: Map<&str, VaultAsset> = Map::new("vault_assets");
// pub pool_address: CanonicalAddr,
//     pub bluna_hub_address: CanonicalAddr,
//     pub bluna_address: CanonicalAddr,