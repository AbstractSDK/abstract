use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for AnsAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.amount)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use speculoos::prelude::*;

    #[test]
    fn test_new() {
        let AnsAsset { name, amount } = AnsAsset::new("crab", 100u128);

        assert_that!(name).is_equal_to(AssetEntry::new("crab"));
        assert_that!(amount).is_equal_to(Uint128::new(100));
    }

    #[test]
    fn test_to_string() {
        let asset = AnsAsset::new("crab", 100u128);

        assert_that!(asset.to_string()).is_equal_to("crab:100".to_string());
    }
}
