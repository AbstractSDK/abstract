use cosmwasm_std::{Addr, CanonicalAddr, Decimal, Uint128};
use cosmwasm_std::{CosmosMsg, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_asset::Asset;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Fee {
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultFee {
    pub flash_loan_fee: Fee,
    pub proxy_fee: Fee,
    pub commission_fee: Fee,
    pub proxy_addr: CanonicalAddr,
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
