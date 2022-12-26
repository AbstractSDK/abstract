




use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::constants::ASSET_DELIMITER;

use crate::objects::AssetEntry;
use cosmwasm_std::StdError;

/// A token that represents Liquidity Pool shares on a dex
/// @todo: move into dex package
#[derive(
    Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema, PartialOrd, Ord, Default,
)]
pub struct LpToken {
    pub dex_name: String,
    pub assets: Vec<String>,
}

const DEX_TO_ASSETS_DELIMITER: &str = "/";

impl TryFrom<AssetEntry> for LpToken {
    type Error = StdError;

    fn try_from(asset: AssetEntry) -> Result<Self, Self::Error> {
        let segments = asset
            .as_str()
            .split(DEX_TO_ASSETS_DELIMITER)
            .collect::<Vec<_>>();

        if segments.len() != 2 {
            return Err(StdError::generic_err(format!(
                "Invalid asset entry: {}",
                asset
            )));
        }

        // get the dex name, like "junoswap"
        let dex_name = segments[0].to_string();

        // get the assets, like "crab,junox" and split them
        let assets: Vec<String> = segments[1]
            .split(ASSET_DELIMITER)
            .map(|s| s.to_string())
            .collect();

        if assets.len() < 2 {
            return Err(StdError::generic_err(format!(
                "Must be at least 2 assets in an LP token: {}",
                asset
            )));
        }

        Ok(Self { dex_name, assets })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use speculoos::prelude::*;

    #[test]
    fn test_from_asset_entry() {
        let lp_token = LpToken::try_from(AssetEntry::new("junoswap/crab,junox")).unwrap();
        assert_that!(lp_token.dex_name).is_equal_to("junoswap".to_string());
        assert_that!(lp_token.assets).is_equal_to(vec!["crab".to_string(), "junox".to_string()]);
    }

    #[test]
    fn test_from_invalid_asset_entry() {
        let lp_token = LpToken::try_from(AssetEntry::new("junoswap/"));
        assert_that!(&lp_token).is_err();
    }

    #[test]
    fn test_fewer_than_two_assets() {
        let lp_token = LpToken::try_from(AssetEntry::new("junoswap/crab"));
        assert_that!(&lp_token).is_err();
    }
}
