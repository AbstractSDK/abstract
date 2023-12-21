use crate::{
    constants::{ASSET_DELIMITER, ATTRIBUTE_DELIMITER, TYPE_DELIMITER},
    objects::{pool_type::PoolType, AssetEntry},
};
use cosmwasm_std::StdError;
use cw_asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

type DexName = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PoolMetadata {
    pub dex: DexName,
    pub pool_type: PoolType,
    pub assets: Vec<AssetEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ResolvedPoolMetadata {
    pub dex: DexName,
    pub pool_type: PoolType,
    pub assets: Vec<AssetInfo>,
}

impl PoolMetadata {
    pub fn new<T: ToString, U: Into<AssetEntry>>(
        dex_name: T,
        pool_type: PoolType,
        assets: Vec<U>,
    ) -> Self {
        let mut assets = assets
            .into_iter()
            .map(|a| a.into())
            .collect::<Vec<AssetEntry>>();
        // sort the asset name
        assets.sort_unstable();
        Self {
            dex: dex_name.to_string(),
            pool_type,
            assets,
        }
    }

    pub fn stable<T: ToString>(dex_name: T, assets: Vec<impl Into<AssetEntry>>) -> Self {
        Self::new(dex_name, PoolType::Stable, assets)
    }

    pub fn weighted<T: ToString>(dex_name: T, assets: Vec<impl Into<AssetEntry>>) -> Self {
        Self::new(dex_name, PoolType::Weighted, assets)
    }

    pub fn constant_product<T: ToString>(dex_name: T, assets: Vec<impl Into<AssetEntry>>) -> Self {
        Self::new(dex_name, PoolType::ConstantProduct, assets)
    }

    pub fn liquidity_bootstrap<T: ToString>(
        dex_name: T,
        assets: Vec<impl Into<AssetEntry>>,
    ) -> Self {
        Self::new(dex_name, PoolType::LiquidityBootstrap, assets)
    }

    pub fn concentrated_liquidity<T: ToString>(
        dex_name: T,
        assets: Vec<impl Into<AssetEntry>>,
    ) -> Self {
        Self::new(dex_name, PoolType::ConcentratedLiquidity, assets)
    }
}

impl FromStr for PoolMetadata {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split it into three parts
        let parts = s.split_once(TYPE_DELIMITER).and_then(|(dex, remainder)| {
            remainder
                .split_once(ATTRIBUTE_DELIMITER)
                .map(|(assets, pool_type)| (dex, assets, pool_type))
        });
        let Some((dex, assets, pool_type)) = parts else {
            return Err(StdError::generic_err(format!(
                "invalid pool metadata format `{s}`; must be in format `{{dex}}{TYPE_DELIMITER}{{asset1}},{{asset2}}{ATTRIBUTE_DELIMITER}{{pool_type}}...`"
            )));
        };

        let assets: Vec<&str> = assets.split(ASSET_DELIMITER).collect();
        let pool_type = PoolType::from_str(pool_type)?;

        Ok(PoolMetadata::new(dex, pool_type, assets))
    }
}

/// To string
/// Ex: "junoswap/uusd,uust:stable"
impl fmt::Display for PoolMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let assets_str = self
            .assets
            .iter()
            .map(|a| a.as_str())
            .collect::<Vec<&str>>()
            .join(ASSET_DELIMITER);
        let pool_type_str = self.pool_type.to_string();
        let dex = &self.dex;

        write!(
            f,
            "{dex}{TYPE_DELIMITER}{assets_str}{ATTRIBUTE_DELIMITER}{pool_type_str}",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speculoos::prelude::*;

    mod implementation {
        use super::*;

        #[test]
        fn new_works() {
            let dex = "junoswap";
            let pool_type = PoolType::Stable;
            let mut assets = vec!["uust".to_string(), "uusd".to_string()];
            let actual = PoolMetadata::new(dex, pool_type, assets.clone());
            // sort the asset names
            assets.sort();
            let expected = PoolMetadata {
                dex: dex.to_string(),
                pool_type,
                assets: assets.into_iter().map(|a| a.into()).collect(),
            };
            assert_that!(actual).is_equal_to(expected);
            assert_that!(actual.to_string()).is_equal_to("junoswap/uusd,uust:stable".to_string());
        }

        #[test]
        fn stable_works() {
            let dex = "junoswap";
            let assets = vec!["uusd".to_string(), "uust".to_string()];
            let actual = PoolMetadata::stable(dex, assets.clone());

            let expected = PoolMetadata {
                dex: dex.to_string(),
                pool_type: PoolType::Stable,
                assets: assets.into_iter().map(|a| a.into()).collect(),
            };
            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn weighted_works() {
            let dex = "junoswap";
            let assets = vec!["uusd".to_string(), "uust".to_string()];
            let actual = PoolMetadata::weighted(dex, assets.clone());

            let expected = PoolMetadata {
                dex: dex.to_string(),
                pool_type: PoolType::Weighted,
                assets: assets.into_iter().map(|a| a.into()).collect(),
            };
            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn constant_product_works() {
            let dex = "junoswap";
            let assets = vec!["uusd".to_string(), "uust".to_string()];
            let actual = PoolMetadata::constant_product(dex, assets.clone());

            let expected = PoolMetadata {
                dex: dex.to_string(),
                pool_type: PoolType::ConstantProduct,
                assets: assets.into_iter().map(|a| a.into()).collect(),
            };
            assert_that!(actual).is_equal_to(expected);
        }

        #[test]
        fn liquidity_bootstrap_works() {
            let dex = "junoswap";
            let assets = vec!["uusd".to_string(), "uust".to_string()];
            let actual = PoolMetadata::liquidity_bootstrap(dex, assets.clone());

            let expected = PoolMetadata {
                dex: dex.to_string(),
                pool_type: PoolType::LiquidityBootstrap,
                assets: assets.into_iter().map(|a| a.into()).collect(),
            };
            assert_that!(actual).is_equal_to(expected);
        }
    }

    #[test]
    fn test_pool_metadata_from_str() {
        let pool_metadata_str = "junoswap/uusd,uust:stable";
        let pool_metadata = PoolMetadata::from_str(pool_metadata_str).unwrap();

        assert_eq!(pool_metadata.dex, "junoswap");
        assert_eq!(
            pool_metadata.assets,
            vec!["uusd", "uust"]
                .into_iter()
                .map(AssetEntry::from)
                .collect::<Vec<AssetEntry>>()
        );
        assert_eq!(pool_metadata.pool_type, PoolType::Stable);

        // Wrong formatting
        let pool_metadata_str = "junoswap:uusd,uust/stable";
        let err = PoolMetadata::from_str(pool_metadata_str).unwrap_err();

        assert_eq!(err, StdError::generic_err(format!(
            "invalid pool metadata format `{pool_metadata_str}`; must be in format `{{dex}}{TYPE_DELIMITER}{{asset1}},{{asset2}}{ATTRIBUTE_DELIMITER}{{pool_type}}...`"
        )));
    }

    #[test]
    fn test_pool_metadata_to_string() {
        let pool_metadata_str = "junoswap/uusd,uust:weighted";
        let pool_metadata = PoolMetadata::from_str(pool_metadata_str).unwrap();

        assert_eq!(pool_metadata.to_string(), pool_metadata_str);
    }
}
