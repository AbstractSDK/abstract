use crate::objects::pool_type::PoolType;
use cosmwasm_std::StdError;

use crate::constants::ASSET_DELIMITER;
use crate::objects::AssetEntry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

type DexName = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PoolMetadata {
    pub dex: DexName,
    pub pool_type: PoolType,
    pub assets: Vec<AssetEntry>,
}

impl PoolMetadata {
    pub fn new<T: ToString, U: Into<AssetEntry>>(
        dex_name: T,
        pool_type: PoolType,
        assets: Vec<U>,
    ) -> Self {
        Self {
            dex: dex_name.to_string(),
            pool_type,
            assets: assets.into_iter().map(|a| Into::into(a)).collect(),
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
}

const ATTRIBUTE_COUNT: usize = 3;
const ATTTRIBUTE_SEPARATOR: &str = ":";

impl FromStr for PoolMetadata {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let attributes: Vec<&str> = s.split(ATTTRIBUTE_SEPARATOR).collect();

        if attributes.len() != ATTRIBUTE_COUNT {
            return Err(StdError::generic_err(format!(
                "invalid pool metadata format `{}`; must be in format `{{dex}}:{{asset1}},{{asset2}}:{{pool_type}}...`",
                s
            )));
        }

        let dex = String::from(attributes[0]);
        let assets: Vec<&str> = attributes[1].split(ASSET_DELIMITER).collect();
        let pool_type = PoolType::from_str(attributes[2])?;

        Ok(PoolMetadata::new(dex, pool_type, assets))
    }
}

/// To string
/// Ex: "junoswap:uusd,uust:stable"
impl fmt::Display for PoolMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let assets_str = self
            .assets
            .iter()
            .map(|a| a.as_str())
            .collect::<Vec<&str>>()
            .join(ASSET_DELIMITER);
        let pool_type_str = self.pool_type.to_string();

        write!(
            f,
            "{}",
            vec![self.dex.clone(), assets_str, pool_type_str].join(ATTTRIBUTE_SEPARATOR)
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
            let assets = vec!["uusd".to_string(), "uust".to_string()];
            let actual = PoolMetadata::new(dex, pool_type.clone(), assets.clone());

            let expected = PoolMetadata {
                dex: dex.to_string(),
                pool_type,
                assets: assets.into_iter().map(|a| a.into()).collect(),
            };
            assert_that!(actual).is_equal_to(expected);
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
        let pool_metadata_str = "junoswap:uusd,uust:stable";
        let pool_metadata = PoolMetadata::from_str(pool_metadata_str).unwrap();

        assert_eq!(pool_metadata.dex, "junoswap");
        assert_eq!(
            pool_metadata.assets,
            vec!["uusd", "uust"]
                .into_iter()
                .map(|a| AssetEntry::from(a))
                .collect::<Vec<AssetEntry>>()
        );
        assert_eq!(pool_metadata.pool_type, PoolType::Stable);
    }

    #[test]
    fn test_pool_metadata_to_string() {
        let pool_metadata_str = "junoswap:uusd,uust:weighted";
        let pool_metadata = PoolMetadata::from_str(pool_metadata_str).unwrap();

        assert_eq!(pool_metadata.to_string(), pool_metadata_str);
    }
}
