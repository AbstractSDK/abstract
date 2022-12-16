use crate::objects::pool_type::PoolType;
use cosmwasm_std::StdError;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

type DexName = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PoolMetadata {
    pub dex: DexName,
    pub pool_type: PoolType,
    pub assets: Vec<String>,
}

const ATTRIBUTE_COUNT: usize = 3;
const ATTTRIBUTE_SEPARATOR: &str = ":";
const ASSET_SEPARATOR: &str = ",";

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
        let assets = String::from(attributes[1])
            .split(ASSET_SEPARATOR)
            .map(String::from)
            .collect();
        let pool_type = PoolType::from_str(attributes[2])?;

        Ok(PoolMetadata {
            dex,
            pool_type,
            assets,
        })
    }
}

/// To string
/// Ex: "junoswap:uusd,uust:stable"
impl fmt::Display for PoolMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let assets_str = self.assets.join(ASSET_SEPARATOR);
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

    #[test]
    fn test_pool_metadata_from_str() {
        let pool_metadata_str = "junoswap:uusd,uust:stable";
        let pool_metadata = PoolMetadata::from_str(pool_metadata_str).unwrap();

        assert_eq!(pool_metadata.dex, "junoswap");
        assert_eq!(pool_metadata.assets, vec!["uusd", "uust"]);
        assert_eq!(pool_metadata.pool_type, PoolType::Stable);
    }

    #[test]
    fn test_pool_metadata_to_string() {
        let pool_metadata_str = "junoswap:uusd,uust:weighted";
        let pool_metadata = PoolMetadata::from_str(pool_metadata_str).unwrap();

        assert_eq!(pool_metadata.to_string(), pool_metadata_str);
    }
}
