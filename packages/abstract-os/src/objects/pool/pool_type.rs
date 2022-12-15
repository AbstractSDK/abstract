use cosmwasm_std::StdError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub enum PoolType {
    Stable,
    Weighted,
    LiquidityBootstrap,
}

impl FromStr for PoolType {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stable" => Ok(PoolType::Stable),
            "weighted" => Ok(PoolType::Weighted),
            "liquidity_bootstrap" => Ok(PoolType::LiquidityBootstrap),
            _ => Err(StdError::generic_err(format!("invalid pool type `{}`", s))),
        }
    }
}

impl TryFrom<String> for PoolType {
    type Error = StdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PoolType::from_str(&value)
    }
}

impl fmt::Display for PoolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolType::Stable => write!(f, "stable"),
            PoolType::Weighted => write!(f, "weighted"),
            PoolType::LiquidityBootstrap => write!(f, "liquidity_bootstrap"),
        }
    }
}
