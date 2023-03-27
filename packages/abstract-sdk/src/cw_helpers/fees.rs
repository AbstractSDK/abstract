//! # Fee helpers
//! Helper trait that lets you easily charge fees on assets

use core::objects::fee::{Fee, UsageFee};
use cosmwasm_std::{CosmosMsg, Uint128};
use cw_asset::Asset;

use crate::AbstractSdkResult;

/// Indicates that the implementing type can be charged fees.
pub trait Chargeable {
    /// Charge a fee on the asset and returns the amount charged.
    fn charge_fee(&mut self, fee: Fee) -> AbstractSdkResult<Uint128>;
    /// Charge a fee on the asset and returns the fee transfer message.
    fn charge_usage_fee(&mut self, fee: UsageFee) -> AbstractSdkResult<Option<CosmosMsg>>;
}

impl Chargeable for Asset {
    fn charge_fee(&mut self, fee: Fee) -> AbstractSdkResult<Uint128> {
        let fee_amount = fee.compute(self.amount);
        self.amount -= fee_amount;
        Ok(fee_amount)
    }

    /// returns a fee message if fee > 0
    fn charge_usage_fee(&mut self, fee: UsageFee) -> AbstractSdkResult<Option<CosmosMsg>> {
        let fee_amount = fee.compute(self.amount);
        if fee_amount.is_zero() {
            return Ok(None);
        }
        self.amount -= fee_amount;
        Ok(Some(
            Asset::new(self.info.clone(), fee_amount).transfer_msg(fee.recipient())?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{testing::MockApi, Addr, Decimal};
    use cw_asset::AssetInfo;

    // test that we can charge fees on assets

    #[test]
    fn test_charge_fee() {
        let info = AssetInfo::native("uusd");
        let mut asset = Asset::new(info, 1000u128);
        let fee = Fee::new(Decimal::percent(10)).unwrap();
        let charged = asset.charge_fee(fee).unwrap();
        assert_eq!(asset.amount.u128(), 900);
        assert_eq!(charged.u128(), 100);
    }
    // test transfer fee
    #[test]
    fn test_charge_transfer_fee() {
        let info = AssetInfo::native("uusd");
        let mut asset: Asset = Asset::new(info.clone(), 1000u128);
        let fee = UsageFee::new(
            &MockApi::default(),
            Decimal::percent(10),
            Addr::unchecked("recipient"),
        )
        .unwrap();
        let msg = asset.charge_usage_fee(fee).unwrap();
        assert_eq!(asset.amount.u128(), 900);
        assert_eq!(
            msg,
            Some(
                Asset::new(info, 100u128)
                    .transfer_msg(Addr::unchecked("recipient"))
                    .unwrap()
            )
        );

        // test zero fee
        let fee = UsageFee::new(
            &MockApi::default(),
            Decimal::zero(),
            Addr::unchecked("recipient"),
        )
        .unwrap();

        let msg = asset.charge_usage_fee(fee).unwrap();
        assert_eq!(asset.amount.u128(), 900);
        assert_eq!(msg, None);
    }
}
