use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::AssetEntry;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AnsAsset {
    pub name: AssetEntry,
    pub amount: Uint128,
}

impl AnsAsset {
    pub fn new(name: impl Into<AssetEntry>, amount: impl Into<Uint128>) -> Self {
        AnsAsset {
            name: name.into(),
            amount: amount.into(),
        }
    }
}
