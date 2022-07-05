use cosmwasm_std::{Addr, Decimal, Uint128};
use cosmwasm_std::{CosmosMsg, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_asset::Asset;

/// A wrapper around Decimal to help handle fractional fees.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Fee {
    /// fraction of asset to take as fee.
    pub share: Decimal,
}

impl Fee {
    pub fn compute(&self, amount: Uint128) -> Uint128 {
        amount * self.share
    }

    pub fn msg(&self, asset: Asset, recipient: Addr) -> StdResult<CosmosMsg> {
        asset.transfer_msg(recipient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee() {
        let fee = Fee {
            share: Decimal::percent(20u64),
        };
        let deposit = Uint128::from(1000000u64);
        let deposit_fee = fee.compute(deposit);
        assert_eq!(deposit_fee, Uint128::from(200000u64));
    }
}
