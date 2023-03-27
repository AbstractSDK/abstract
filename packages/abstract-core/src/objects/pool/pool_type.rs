use cosmwasm_std::StdError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub enum PoolType {
    ConstantProduct,
    Stable,
    Weighted,
    LiquidityBootstrap,
}

const CONSTANT_PRODUCT: &str = "constant_product";
const STABLE: &str = "stable";
const WEIGHTED: &str = "weighted";
const LIQUIDITY_BOOTSTRAP: &str = "liquidity_bootstrap";

impl FromStr for PoolType {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            CONSTANT_PRODUCT => Ok(PoolType::ConstantProduct),
            STABLE => Ok(PoolType::Stable),
            WEIGHTED => Ok(PoolType::Weighted),
            LIQUIDITY_BOOTSTRAP => Ok(PoolType::LiquidityBootstrap),
            _ => Err(StdError::generic_err(format!("invalid pool type `{s}`"))),
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
            PoolType::ConstantProduct => write!(f, "{CONSTANT_PRODUCT}"),
            PoolType::Stable => write!(f, "{STABLE}"),
            PoolType::Weighted => write!(f, "{WEIGHTED}"),
            PoolType::LiquidityBootstrap => write!(f, "{LIQUIDITY_BOOTSTRAP}"),
        }
    }
}
