use crate::{
    constants::{ASSET_DELIMITER, TYPE_DELIMITER},
    objects::AssetEntry,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub type DexName = String;

/// A key for the token that represents Liquidity Pool shares on a dex
/// Will be formatted as "dex_name/asset1,asset2" when serialized
#[derive(
    Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord, Default,
)]
pub struct LpToken {
    pub dex: DexName,
    pub assets: Vec<AssetEntry>,
}

impl LpToken {
    pub fn new<T: ToString, U: Into<AssetEntry> + Clone>(dex_name: T, assets: Vec<U>) -> Self {
        let mut assets = assets
            .into_iter()
            .map(|a| a.into())
            .collect::<Vec<AssetEntry>>();
        // sort the asset name
        assets.sort_unstable();
        Self {
            dex: dex_name.to_string(),
            assets,
        }
    }
}

/// Transform into a string formatted as "dex_name/asset1,asset2"
impl Display for LpToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let assets = self
            .assets
            .iter()
            .map(|a| a.as_str())
            .collect::<Vec<&str>>()
            .join(ASSET_DELIMITER);

        write!(f, "{}{}{}", self.dex, TYPE_DELIMITER, assets)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use speculoos::prelude::*;

    mod implementation {
        use super::*;

        #[test]
        fn new_works() {
            let dex_name = "junoswap";
            let mut assets = vec![AssetEntry::from("junox"), AssetEntry::from("crab")];
            let actual = LpToken::new(dex_name, assets.clone());
            assets.sort();
            let expected = LpToken {
                dex: dex_name.to_string(),
                assets,
            };
            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn assets_returns_asset_entries() {
            let dex_name = "junoswap";
            let assets = vec![AssetEntry::from("crab"), AssetEntry::from("junox")];
            let lp_token = LpToken::new(dex_name, assets);
            let expected = vec![AssetEntry::from("crab"), AssetEntry::from("junox")];

            assert_that!(lp_token.assets).is_equal_to(expected);
        }
    }

    mod from_asset_entry {
        use crate::objects::AnsEntryConvertor;

        use super::*;

        #[test]
        fn test_from_asset_entry() {
            let asset = AssetEntry::new("junoswap/crab,junox");
            let lp_token = AnsEntryConvertor::new(asset).lp_token().unwrap();
            assert_that!(lp_token.dex).is_equal_to("junoswap".to_string());
            assert_that!(lp_token.assets)
                .is_equal_to(vec![AssetEntry::from("crab"), AssetEntry::from("junox")]);
        }

        #[test]
        fn test_from_invalid_asset_entry() {
            let asset = AssetEntry::new("junoswap/");
            let lp_token = AnsEntryConvertor::new(asset).lp_token();
            assert_that!(&lp_token).is_err();
        }

        #[test]
        fn test_fewer_than_two_assets() {
            let asset = AssetEntry::new("junoswap/crab");
            let lp_token = AnsEntryConvertor::new(asset).lp_token();
            assert_that!(&lp_token).is_err();
        }
    }

    mod into_asset_entry {
        use crate::objects::AnsEntryConvertor;

        use super::*;

        #[test]
        fn into_asset_entry_works() {
            let lp_token = LpToken::new("junoswap", vec!["crab".to_string(), "junox".to_string()]);
            let expected = AssetEntry::new("junoswap/crab,junox");

            assert_that!(AnsEntryConvertor::new(lp_token).asset_entry()).is_equal_to(expected);
        }
    }

    mod from_pool_metadata {
        use super::*;
        use crate::objects::{AnsEntryConvertor, PoolMetadata, PoolType};

        #[test]
        fn test_from_pool_metadata() {
            let assets: Vec<AssetEntry> = vec!["crab".into(), "junox".into()];
            let dex = "junoswap".to_string();

            let pool = PoolMetadata {
                dex: dex.clone(),
                pool_type: PoolType::Stable,
                assets: assets.clone(),
            };

            let lp_token = AnsEntryConvertor::new(pool).lp_token();
            assert_that!(lp_token.dex).is_equal_to(dex);
            assert_that!(lp_token.assets).is_equal_to(assets);
        }
    }
}
