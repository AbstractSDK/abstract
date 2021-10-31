use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Deposit {
    value: Uint128,
}

impl Deposit {
    pub const fn new() -> Deposit {
        Deposit {
            value: Uint128::zero(),
        }
    }

    pub fn increase(&mut self, amount: Uint128) -> Self {
        self.value += amount;
        self.clone()
    }

    pub fn decrease(&mut self, amount: Uint128) -> StdResult<Self> {
        if amount > self.value {
            return Err(StdError::generic_err(format!(
                "Cannot decrease {} by {}",
                self.value, amount
            )));
        }

        self.value -= amount;
        Ok(self.clone())
    }

    pub fn get(&self) -> Uint128 {
        self.value
    }
}

pub struct UserDeposit<'a> {
    map: Map<'a, &'a [u8], Deposit>,
}

impl UserDeposit<'_> {
    pub const fn new() -> UserDeposit<'static> {
        UserDeposit {
            map: Map::new("deposits"),
        }
    }

    pub fn increase(
        &self,
        storage: &mut dyn Storage,
        key: &[u8],
        amount: Uint128,
    ) -> StdResult<()> {
        let user_deposit = &mut self.map.may_load(storage, key)?.unwrap_or_default();
        self.map
            .save(storage, key, &user_deposit.increase(amount))?;
        Ok(())
    }

    pub fn decrease(
        &self,
        storage: &mut dyn Storage,
        key: &[u8],
        amount: Uint128,
    ) -> StdResult<()> {
        let mut user_deposit: Deposit = self.map.may_load(storage, key)?.unwrap_or_default();
        self.map
            .save(storage, key, &user_deposit.decrease(amount)?)?;
        let new_deposit = user_deposit.get();
        if new_deposit == Uint128::zero() {
            self.map.remove(storage, key);
        }
        Ok(())
    }

    pub fn get(&self, storage: &dyn Storage, key: &[u8]) -> StdResult<Uint128> {
        Ok(self.map.may_load(storage, key)?.unwrap_or_default().get())
    }
}

pub struct DepositManager {
    total_deposits: Item<'static, Deposit>,
    user_deposits: UserDeposit<'static>, // TODO: Check if lifetime can be improved
}

impl DepositManager {
    pub const fn new() -> Self {
        Self {
            total_deposits: Item::new("\u{0}{5}total"),
            user_deposits: UserDeposit::new(),
        }
    }

    pub fn increase(
        &self,
        storage: &mut dyn Storage,
        key: &[u8],
        amount: Uint128,
    ) -> StdResult<()> {
        let deposit = self.total_deposits.load(storage);
        if deposit.is_err() {
            println!("new deposit storage");
            self.total_deposits.save(storage, &Deposit::new())?;
        }
        let mut total_deposits = self.total_deposits.load(storage)?;
        self.total_deposits
            .save(storage, &total_deposits.increase(amount))?;
        self.user_deposits.increase(storage, key, amount)
    }

    pub fn decrease(
        &self,
        storage: &mut dyn Storage,
        key: &[u8],
        amount: Uint128,
    ) -> StdResult<()> {
        self.user_deposits.decrease(storage, key, amount)?;
        let mut total_deposits = self.total_deposits.load(storage)?;
        self.total_deposits
            .save(storage, &total_deposits.decrease(amount)?)?;
        Ok(())
    }

    pub fn get(&self, storage: &dyn Storage, key: &[u8]) -> StdResult<Uint128> {
        self.user_deposits.get(storage, key)
    }

    pub fn get_total_deposits(&self, storage: &dyn Storage) -> StdResult<Uint128> {
        Ok(self.total_deposits.load(storage)?.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::MockStorage;

    #[test]
    fn test_user_deposits() {
        let mut storage = MockStorage::default();
        let user_deposits = UserDeposit::new();

        let key = "key".as_bytes();
        let initial_value = user_deposits.get(&mut storage, key).unwrap();
        assert_eq!(initial_value, Uint128::from(0u64));

        user_deposits
            .increase(&mut storage, key, Uint128::from(10u64))
            .unwrap();
        let value = user_deposits.get(&mut storage, key).unwrap();
        assert_eq!(value, Uint128::from(10u64));

        user_deposits
            .increase(&mut storage, key, Uint128::from(10u64))
            .unwrap();
        let value = user_deposits.get(&mut storage, key).unwrap();
        assert_eq!(value, Uint128::from(20u64));

        user_deposits
            .decrease(&mut storage, key, Uint128::from(5u64))
            .unwrap();
        let value = user_deposits.get(&mut storage, key).unwrap();
        assert_eq!(value, Uint128::from(15u64));

        user_deposits
            .decrease(&mut storage, key, Uint128::from(15u64))
            .unwrap();
        let value = user_deposits.get(&mut storage, key).unwrap();
        assert_eq!(value, Uint128::from(0u64));

        let res = user_deposits.decrease(&mut storage, key, Uint128::from(15u64));
        assert!(res.is_err());
    }

    #[test]
    fn test_deposit_manager() {
        let mut storage = MockStorage::default();
        let deposits = DepositManager::new();

        let key = "key".as_bytes();
        let other_key = "other_key".as_bytes();
        let initial_value = deposits.get(&mut storage, key).unwrap();
        assert_eq!(initial_value, Uint128::from(0u64));

        deposits
            .increase(&mut storage, key, Uint128::from(10u64))
            .unwrap();
        let value = deposits.get(&mut storage, key).unwrap();
        assert_eq!(value, Uint128::from(10u64));
        let value = deposits.get_total_deposits(&storage).unwrap();
        assert_eq!(value, Uint128::from(10u64));

        deposits
            .increase(&mut storage, key, Uint128::from(10u64))
            .unwrap();
        let value = deposits.get(&mut storage, key).unwrap();
        assert_eq!(value, Uint128::from(20u64));
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint128::from(20u64)
        );

        deposits
            .increase(&mut storage, other_key, Uint128::from(10u64))
            .unwrap();
        let value = deposits.get(&mut storage, key).unwrap();
        assert_eq!(value, Uint128::from(20u64));
        let value = deposits.get(&mut storage, other_key).unwrap();
        assert_eq!(value, Uint128::from(10u64));
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint128::from(30u64)
        );

        let res = deposits.decrease(&mut storage, other_key, Uint128::from(15u64));
        assert!(res.is_err());

        deposits
            .decrease(&mut storage, key, Uint128::from(15u64))
            .unwrap();
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint128::from(15u64)
        );

        deposits
            .decrease(&mut storage, key, Uint128::from(5u64))
            .unwrap();
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint128::from(10u64)
        );

        deposits
            .decrease(&mut storage, other_key, Uint128::from(10u64))
            .unwrap();
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint128::from(0u64)
        );
    }
}
