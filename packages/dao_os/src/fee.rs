use cosmwasm_std::{Addr, CanonicalAddr, Decimal, Uint128};
use cosmwasm_std::{CosmosMsg, Deps, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use terraswap::asset::Asset;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Fee {
    pub share: Decimal,
}

impl Fee {
    pub fn compute(&self, amount: Uint128) -> Uint128 {
        amount * self.share
    }

    pub fn msg(&self, deps: Deps, asset: Asset, recipient: Addr) -> StdResult<CosmosMsg> {
        asset.into_msg(&deps.querier, recipient)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultFee {
    pub flash_loan_fee: Fee,
    pub treasury_fee: Fee,
    pub commission_fee: Fee,
    pub treasury_addr: CanonicalAddr,
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
