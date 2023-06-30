use crate::{error::AbstractError, AbstractResult};
use cosmwasm_std::{Storage, Uint64};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Deposit {
    value: Uint64,
}

impl Deposit {
    pub const fn new() -> Deposit {
        Deposit {
            value: Uint64::zero(),
        }
    }

    pub fn increase(&mut self, amount: Uint64) -> Self {
        self.value += amount;
        self.clone()
    }

    pub fn decrease(&mut self, amount: Uint64) -> AbstractResult<Self> {
        if amount > self.value {
            return Err(AbstractError::Deposit(format!(
                "Cannot decrease {} by {}",
                self.value, amount
            )));
        }

        self.value = self.value.checked_sub(amount)?;
        Ok(self.clone())
    }

    pub fn get(&self) -> Uint64 {
        self.value
    }
}

pub struct UserDeposit<'a> {
    map: Map<'a, &'a [u8], Deposit>,
}

impl UserDeposit<'_> {
    pub const fn new(namespace: &'static str) -> UserDeposit<'static> {
        UserDeposit {
            map: Map::new(namespace),
        }
    }

    pub fn increase(
        &self,
        storage: &mut dyn Storage,
        key: &[u8],
        amount: Uint64,
    ) -> AbstractResult<()> {
        let user_deposit = &mut self.map.may_load(storage, key)?.unwrap_or_default();
        self.map
            .save(storage, key, &user_deposit.increase(amount))?;
        Ok(())
    }

    pub fn decrease(
        &self,
        storage: &mut dyn Storage,
        key: &[u8],
        amount: Uint64,
    ) -> AbstractResult<()> {
        let mut user_deposit: Deposit = self.map.may_load(storage, key)?.unwrap_or_default();
        self.map
            .save(storage, key, &user_deposit.decrease(amount)?)?;
        let new_deposit = user_deposit.get();
        if new_deposit == Uint64::zero() {
            self.map.remove(storage, key);
        }
        Ok(())
    }

    pub fn get(&self, storage: &dyn Storage, key: &[u8]) -> AbstractResult<Uint64> {
        Ok(self.map.may_load(storage, key)?.unwrap_or_default().get())
    }
}

pub struct DepositManager {
    total_deposits: Item<'static, Deposit>,
    user_deposits: UserDeposit<'static>, // TODO: Check if lifetime can be improved
}

impl DepositManager {
    pub const fn new(total_namespace: &'static str, deposit_namespace: &'static str) -> Self {
        Self {
            total_deposits: Item::new(total_namespace),
            user_deposits: UserDeposit::new(deposit_namespace),
        }
    }

    pub fn increase(
        &self,
        storage: &mut dyn Storage,
        key: &[u8],
        amount: Uint64,
    ) -> AbstractResult<()> {
        let deposit = self.total_deposits.load(storage);
        if deposit.is_err() {
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
        amount: Uint64,
    ) -> AbstractResult<()> {
        self.user_deposits.decrease(storage, key, amount)?;
        let mut total_deposits = self.total_deposits.load(storage)?;
        self.total_deposits
            .save(storage, &total_deposits.decrease(amount)?)?;
        Ok(())
    }

    pub fn get(&self, storage: &dyn Storage, key: &[u8]) -> AbstractResult<Uint64> {
        self.user_deposits.get(storage, key)
    }

    pub fn get_total_deposits(&self, storage: &dyn Storage) -> AbstractResult<Uint64> {
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
        let user_deposits = UserDeposit::new("test");

        let key = "key".as_bytes();
        let initial_value = user_deposits.get(&storage, key).unwrap();
        assert_eq!(initial_value, Uint64::from(0u64));

        user_deposits
            .increase(&mut storage, key, Uint64::from(10u64))
            .unwrap();
        let value = user_deposits.get(&storage, key).unwrap();
        assert_eq!(value, Uint64::from(10u64));

        user_deposits
            .increase(&mut storage, key, Uint64::from(10u64))
            .unwrap();
        let value = user_deposits.get(&storage, key).unwrap();
        assert_eq!(value, Uint64::from(20u64));

        user_deposits
            .decrease(&mut storage, key, Uint64::from(5u64))
            .unwrap();
        let value = user_deposits.get(&storage, key).unwrap();
        assert_eq!(value, Uint64::from(15u64));

        user_deposits
            .decrease(&mut storage, key, Uint64::from(15u64))
            .unwrap();
        let value = user_deposits.get(&storage, key).unwrap();
        assert_eq!(value, Uint64::from(0u64));

        let res = user_deposits.decrease(&mut storage, key, Uint64::from(15u64));
        assert!(res.is_err());
    }

    #[test]
    fn test_deposit_manager() {
        let mut storage = MockStorage::default();
        let deposits = DepositManager::new("test", "test2");

        let key = "key".as_bytes();
        let other_key = "other_key".as_bytes();
        let initial_value = deposits.get(&storage, key).unwrap();
        assert_eq!(initial_value, Uint64::from(0u64));

        deposits
            .increase(&mut storage, key, Uint64::from(10u64))
            .unwrap();
        let value = deposits.get(&storage, key).unwrap();
        assert_eq!(value, Uint64::from(10u64));
        let value = deposits.get_total_deposits(&storage).unwrap();
        assert_eq!(value, Uint64::from(10u64));

        deposits
            .increase(&mut storage, key, Uint64::from(10u64))
            .unwrap();
        let value = deposits.get(&storage, key).unwrap();
        assert_eq!(value, Uint64::from(20u64));
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint64::from(20u64)
        );

        deposits
            .increase(&mut storage, other_key, Uint64::from(10u64))
            .unwrap();
        let value = deposits.get(&storage, key).unwrap();
        assert_eq!(value, Uint64::from(20u64));
        let value = deposits.get(&storage, other_key).unwrap();
        assert_eq!(value, Uint64::from(10u64));
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint64::from(30u64)
        );

        let res = deposits.decrease(&mut storage, other_key, Uint64::from(15u64));
        assert!(res.is_err());

        deposits
            .decrease(&mut storage, key, Uint64::from(15u64))
            .unwrap();
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint64::from(15u64)
        );

        deposits
            .decrease(&mut storage, key, Uint64::from(5u64))
            .unwrap();
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint64::from(10u64)
        );

        deposits
            .decrease(&mut storage, other_key, Uint64::from(10u64))
            .unwrap();
        assert_eq!(
            deposits.get_total_deposits(&storage).unwrap(),
            Uint64::from(0u64)
        );
    }
}
