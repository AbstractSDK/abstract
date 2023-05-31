use crate::{error::AbstractError, AbstractResult};
use cosmwasm_std::{Addr, Api, Coin, CosmosMsg, Decimal, MessageInfo, Uint128};
use cw_asset::Asset;

/// A wrapper around Fee to help handle fee logic.
/// Use this with `Chargeable` trait in the SDK to charge fees on asset structs.
#[cosmwasm_schema::cw_serde]
pub struct UsageFee {
    fee: Fee,
    recipient: Addr,
}

impl UsageFee {
    pub fn new(
        api: &dyn Api,
        share: Decimal,
        recipient: impl Into<String>,
    ) -> AbstractResult<Self> {
        let recipient = api.addr_validate(&recipient.into())?;
        let fee = Fee::new(share)?;
        Ok(UsageFee { fee, recipient })
    }
    pub fn set_share(&mut self, share: Decimal) -> AbstractResult<()> {
        self.fee = Fee::new(share)?;
        Ok(())
    }
    pub fn share(&self) -> Decimal {
        self.fee.share()
    }
    pub fn compute(&self, amount: Uint128) -> Uint128 {
        amount * self.share()
    }
    pub fn recipient(&self) -> Addr {
        self.recipient.clone()
    }
    pub fn set_recipient(
        &mut self,
        api: &dyn Api,
        recipient: impl Into<String>,
    ) -> AbstractResult<()> {
        self.recipient = api.addr_validate(&recipient.into())?;
        Ok(())
    }
}

/// A wrapper around Decimal to help handle fractional fees.
#[cosmwasm_schema::cw_serde]
pub struct Fee {
    /// fraction of asset to take as fee.
    share: Decimal,
}

impl Fee {
    pub fn new(share: Decimal) -> AbstractResult<Self> {
        if share >= Decimal::percent(100) {
            return Err(AbstractError::Fee(
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

/// A wrapper around Coin to help handle fixed fees (with multiples).
#[cosmwasm_schema::cw_serde]
pub struct FixedFee {
    /// Fee to be paid for a unit of a service
    fee: Coin,
}

impl FixedFee {
    /// Creates a wrapped coin to allow charging a fee
    pub fn new(fee: &Coin) -> Self {
        FixedFee { fee: fee.clone() }
    }
    /// Allows to collect the fee multiple times
    /// E.g., for namespaces, you want to charge the number of claimed namespaces times the fee for 1 namespace
    pub fn quantity(mut self, qty: u128) -> Self {
        self.fee.amount *= Uint128::from(qty);
        self
    }

    /// Validates that the sent funds correspond exactly to the fixed fee
    /// Returns the fee object (a.k.a. self) for later use (e.g. transferring the paid fee to another address)
    pub fn assert_payment(self, msg_info: &MessageInfo) -> AbstractResult<Coin> {
        if self.fee.amount.is_zero() {
            return Ok(self.fee);
        }
        if msg_info.funds.len() != 1
            || msg_info.funds[0].denom != self.fee.denom
            || self.fee.amount != msg_info.funds[0].amount
        {
            return Err(AbstractError::Fee(format!(
                "Invalid fee payment sent. Expected {}, sent {:?}",
                self.fee, msg_info.funds
            )));
        }
        Ok(self.fee)
    }

    /// Validates that the sent funds include at least the fixed fee
    /// This mutates the msg_info so that the rest of the message execution doesn't include those funds anymore.
    /// This acts as a toll on the sent funds
    /// Returns the fee object (a.k.a. self) for later use (e.g. transferring the paid fee to another address)
    pub fn charge(self, msg_info: &mut MessageInfo) -> AbstractResult<Coin> {
        if self.fee.amount.is_zero() {
            return Ok(self.fee);
        }
        let original_funds = msg_info.funds.clone();

        // We find the fee inside the msg_info
        let funds_to_use = msg_info
            .funds
            .iter_mut()
            .find(|f| f.denom == self.fee.denom)
            .ok_or(AbstractError::Fee(format!(
                "Invalid fee payment sent. Expected {}, sent {:?}",
                self.fee, original_funds
            )))?;

        if funds_to_use.amount < self.fee.amount {
            return Err(AbstractError::Fee(format!(
                "Invalid fee payment sent. Expected {}, sent {:?}",
                self.fee, original_funds
            )));
        }

        funds_to_use.amount -= self.fee.amount;
        Ok(self.fee)
    }
    pub fn fee(&self) -> Coin {
        self.fee.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fee {
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
    mod transfer_fee {
        use cosmwasm_std::{
            coin, coins,
            testing::{mock_info, MockApi},
        };

        use super::*;

        #[test]
        fn test_transfer_fee_new() {
            let api = MockApi::default();
            let fee = UsageFee::new(&api, Decimal::percent(20u64), "recipient").unwrap();
            let deposit = Uint128::from(1000000u64);
            let deposit_fee = fee.compute(deposit);
            assert_eq!(deposit_fee, Uint128::from(200000u64));
        }

        #[test]
        fn test_transfer_fee_share() {
            let api = MockApi::default();
            let expected_percent = 20u64;
            let fee = UsageFee::new(&api, Decimal::percent(expected_percent), "recipient").unwrap();
            assert_eq!(fee.share(), Decimal::percent(expected_percent));
        }

        #[test]
        fn test_transfer_fee_msg() {
            let api = MockApi::default();
            let fee = UsageFee::new(&api, Decimal::percent(20u64), "recipient").unwrap();
            let asset = Asset::native("uusd", Uint128::from(1000000u64));

            let recipient = Addr::unchecked("recipient");
            let msg = fee.fee.msg(asset.clone(), recipient.clone()).unwrap();
            assert_eq!(msg, asset.transfer_msg(recipient).unwrap(),);
        }

        #[test]
        fn test_transfer_fee_new_gte_100() {
            let api = MockApi::default();
            let fee = UsageFee::new(&api, Decimal::percent(100u64), "recipient");
            assert!(fee.is_err());
            let fee = UsageFee::new(&api, Decimal::percent(101u64), "recipient");
            assert!(fee.is_err());
        }

        #[test]
        fn test_transfer_fee_set_recipient() {
            let api = MockApi::default();
            let mut fee = UsageFee::new(&api, Decimal::percent(20u64), "recipient").unwrap();
            let new_recipient = "new_recipient";
            fee.set_recipient(&api, new_recipient).unwrap();
            assert_eq!(fee.recipient(), Addr::unchecked(new_recipient));
        }
        #[test]
        fn test_transfer_fee_set_share() {
            let api = MockApi::default();
            let mut fee = UsageFee::new(&api, Decimal::percent(20u64), "recipient").unwrap();
            let new_share = Decimal::percent(10u64);
            fee.set_share(new_share).unwrap();
            assert_eq!(fee.share(), new_share);
        }
        #[test]
        fn test_loose_fee_validation() {
            let _api = MockApi::default();
            let fee = FixedFee::new(&coin(45, "ujunox"));
            let mut info = mock_info("anyone", &coins(47, "ujunox"));
            fee.charge(&mut info).unwrap();
            assert_eq!(info.funds, coins(2, "ujunox"));
        }
    }
}
