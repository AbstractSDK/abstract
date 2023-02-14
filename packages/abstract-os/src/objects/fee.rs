use cosmwasm_std::{Addr, CosmosMsg, Decimal, Uint128};
use cw_asset::Asset;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{error::AbstractOsError, AbstractResult};

/// A wrapper around Decimal to help handle fractional fees.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Fee {
    /// fraction of asset to take as fee.
    share: Decimal,
}

impl Fee {
    pub fn new(share: Decimal) -> AbstractResult<Self> {
        if share >= Decimal::percent(100) {
            return Err(AbstractOsError::Fee(
                "fee share must be lesser than 100%".to_string(),
            ));
        }
        Ok(Fee { share })
    }
    pub fn compute(&self, amount: Uint128) -> Uint128 {
        amount * self.share
    }

    pub fn msg(&self, asset: Asset, recipient: Addr) -> AbstractResult<CosmosMsg> {
        asset.transfer_msg(recipient).map_err(Into::into)
    }
    pub fn share(&self) -> Decimal {
        self.share
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_manual_construction() {
        let fee = Fee {
            share: Decimal::percent(20u64),
        };
        let deposit = Uint128::from(1000000u64);
        let deposit_fee = fee.compute(deposit);
        assert_eq!(deposit_fee, Uint128::from(200000u64));
    }

    #[test]
    fn test_fee_new() {
        let fee = Fee::new(Decimal::percent(20u64)).unwrap();
        let deposit = Uint128::from(1000000u64);
        let deposit_fee = fee.compute(deposit);
        assert_eq!(deposit_fee, Uint128::from(200000u64));
    }

    #[test]
    fn test_fee_new_gte_100() {
        let fee = Fee::new(Decimal::percent(100u64));
        assert!(fee.is_err());
        let fee = Fee::new(Decimal::percent(101u64));
        assert!(fee.is_err());
    }

    #[test]
    fn test_fee_share() {
        let expected_percent = 20u64;
        let fee = Fee::new(Decimal::percent(expected_percent)).unwrap();
        assert_eq!(fee.share(), Decimal::percent(expected_percent));
    }

    #[test]
    fn test_fee_msg() {
        let fee = Fee::new(Decimal::percent(20u64)).unwrap();
        let asset = Asset::native("uusd", Uint128::from(1000000u64));

        let recipient = Addr::unchecked("recipient");
        let msg = fee.msg(asset.clone(), recipient.clone()).unwrap();
        assert_eq!(msg, asset.transfer_msg(recipient).unwrap(),);
    }
}
