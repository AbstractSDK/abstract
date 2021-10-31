use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::Item;

use crate::vault_info::VaultInfoRaw;

pub static LUNA_DENOM: &str = "uluna";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
    pub trader: CanonicalAddr,
}

// Allowances
pub const STATE: Item<State> = Item::new("\u{0}{5}state");

// Vault information bluna_address: CanonicalAddr,
pub const VAULT_INFO: Item<VaultInfoRaw> = Item::new("\u{0}{5}vault");