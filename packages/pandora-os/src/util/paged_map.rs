use cosmwasm_std::{CosmosMsg, Deps, DepsMut, Empty, Order, StdError, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map, PrimaryKey, Path};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::deposit_manager::Deposit;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;
const MAX_MSG_LIMIT: u32 = 15;

pub type PaginationResult<A, PageResult> = StdResult<(Option<A>, PageResult)>;
/// Allows for multi-transaction computation on a dataset. Required for large datasets due to gas constraints.
pub struct PagedMap<'a, T, R> {
    /// Actual data store
    data: Map<'a, &'a [u8], T>,
    /// Pagination progress status
    pub status: Item<'a, PaginationInfo<R>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationInfo<R> {
    /// Prevents map manipulation during pagination
    pub is_locked: bool,
    /// Starting item for next iteration
    pub last_processed_item: Option<Vec<u8>>,
    /// Accumulator item available for use in pagination function
    pub accumulator: Option<R>,
}

impl<'a, T, R> PagedMap<'a, T, R> {
    pub const fn new(namespace: &'a str, status_namespace: &'a str) -> Self {
        PagedMap {
            data: Map::new(namespace),
            status: Item::new(status_namespace),
        }
    }

    pub fn instantiate(&self, store: &mut dyn Storage) -> Result<(), StdError>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.status.save(
            store,
            &PaginationInfo {
                is_locked: false,
                accumulator: None,
                last_processed_item: None,
            },
        )
    }

    pub fn save(&self, store: &mut dyn Storage, key: &[u8], data: &T) -> StdResult<()>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        if self.status.load(store)?.is_locked {
            return Err(StdError::GenericErr {
                msg: "Can not save to map while locked. Proceed with operation first.".into(),
            });
        }
        self.data.save(store, key, data)
    }

    /// **Warning**: This function circumvents the storage lock. You should only use this in a pagination function.
    pub fn unsafe_save(&self, store: &mut dyn Storage, key: &[u8], data: &T) -> StdResult<()>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.save(store, key, data)
    }

    // Returns the removed item after deleting it
    pub fn remove(&self, store: &mut dyn Storage, key: &[u8]) -> StdResult<T>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        if self.status.load(store)?.is_locked {
            return Err(StdError::GenericErr {
                msg: "Can not save to map while locked. Proceed with operation first.".into(),
            });
        }
        let old_item = self.data.load(store, key)?;
        self.data.remove(store, key);

        Ok(old_item)
    }

    /// **Warning**: This function circumvents the storage lock. You should only use this in a pagination function.
    /// Returns the removed item after deleting it
    pub fn unsafe_remove(&self, store: &mut dyn Storage, key: &[u8]) -> StdResult<T>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        let old_item = self.data.load(store, key)?;
        self.data.remove(store, key);

        Ok(old_item)
    }

    pub fn load(&self, store: &dyn Storage, key: &[u8]) -> StdResult<T>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.load(store, key)
    }

    pub fn has(&self, store: &dyn Storage, key: &[u8]) -> bool
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.has(store, key)
    }

    pub fn may_load(&self, store: &dyn Storage, key: &[u8]) -> StdResult<Option<T>>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.may_load(store, key)
    }

    pub fn load_status(&self, store: &dyn Storage) -> StdResult<PaginationInfo<R>>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.status.load(store)
    }

    pub fn key(&self, key: &[u8]) -> Path<T>
    where
        T: Serialize + DeserializeOwned,
    {
        self.data.key(key)
    }

    /// Perform some operation on a page of the map.
    /// Returns an optional result of that computation.
    /// Repeat until state unlocks to page over the whole map
    /// Omits errors from f()
    pub fn page_with_accumulator<C, FuncResult>(
        &self,
        deps: DepsMut,
        limit: Option<u32>,
        context: &C,
        f: fn(&[u8], &mut dyn Storage, T, &mut R, &C) -> StdResult<Option<FuncResult>>,
    ) -> PaginationResult<R, Vec<FuncResult>>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let mut status = self.status.load(deps.storage)?;
        if status.accumulator.is_none() {
            status.is_locked = true;
            status.accumulator = Some(R::default());
            status.last_processed_item = None;
        }

        let start = status.last_processed_item.clone().map(Bound::exclusive);

        let result: Vec<(Vec<u8>, T)> = self
            .data
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .collect::<StdResult<Vec<(Vec<u8>, T)>>>()?;

        // If not all items processed, update last item
        let return_accumulator = if !result.is_empty() {
            let last_key = result.last().unwrap().0.clone();
            status.last_processed_item = Some(last_key);
            None
        } else {
            // Everything processed, set to None and return accumulator
            let accumulator: Option<R> = status.accumulator.clone();
            status.is_locked = false;
            status.accumulator = None;
            accumulator
        };

        let function_results = result
            .into_iter()
            .filter_map(|(key, element)| {
                f(
                    &key,
                    deps.storage,
                    element,
                    status
                        .accumulator
                        .as_mut()
                        .expect("accumulator contains some value"),
                    context,
                )
                .ok()
                .unwrap_or(None)
            })
            .collect::<Vec<FuncResult>>();

        self.status.save(deps.storage, &status)?;

        Ok((return_accumulator, function_results))
    }

    /// Will apply function on each element (key, value) of the map. Errors on function f() are neglected.
    pub fn page_for_msgs<C>(
        &self,
        deps: DepsMut,
        limit: Option<u32>,
        context: &C,
        f: fn(
            (Vec<u8>, &mut T),
            &mut dyn Storage,
            context: &C,
        ) -> StdResult<Option<CosmosMsg<Empty>>>,
    ) -> StdResult<Option<Vec<CosmosMsg<Empty>>>>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_MSG_LIMIT) as usize;
        let mut status = self.status.load(deps.storage)?;
        if status.last_processed_item.is_none() {
            status.is_locked = true;
        }

        let start = status.last_processed_item.clone().map(Bound::exclusive);
        let store = deps.as_ref().storage;

        let result: Vec<(Vec<u8>, T)> = self
            .data
            .range(store, start, None, Order::Ascending)
            .take(limit)
            .collect::<StdResult<Vec<(Vec<u8>, T)>>>()?;

        // Not all items processed, update last item
        let maybe_msgs = if result.len() != 0 {
            status.last_processed_item = result.last().map(|(key, _)| key.clone());
            Some(
                result
                    .into_iter()
                    .flat_map(|(key, mut item)| f((key, &mut item), deps.storage, context).ok())
                    .flatten()
                    .collect(),
            )
        } else {
            // Everything is processed, unlock and set last item to None
            status.is_locked = false;
            status.last_processed_item = None;
            None
        };

        self.status.save(deps.storage, &status)?;
        Ok(maybe_msgs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::ops::Deref;

    #[cfg(feature = "iterator")]
    use cosmwasm_storage::iter_helpers::to_length_prefixed;
    use cw_storage_plus::U8Key;
    use cosmwasm_std::testing::{MockStorage, mock_dependencies};
    #[cfg(feature = "iterator")]
    use cosmwasm_std::{Order, StdResult};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Data {
        pub name: String,
        pub balance: u32,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
    struct IncomeAcc {
        pub total: u32,
    }

    const USERS: PagedMap<Data,IncomeAcc> = PagedMap::new("people","status");

    #[test]
    fn save_and_load() {
        let mut store = MockStorage::new();

        // save and load on one key
        let john = USERS.key(b"john");
        let data = Data {
            name: "John".to_string(),
            balance: 32,
        };
        assert_eq!(None, john.may_load(&store).unwrap());
        john.save(&mut store, &data).unwrap();
        assert_eq!(data, john.load(&store).unwrap());

        // nothing on another key
        assert_eq!(None, USERS.may_load(&store, b"jack").unwrap());

        // same named path gets the data
        assert_eq!(data, USERS.load(&store, b"john").unwrap());

        // removing leaves us empty
        john.remove(&mut store);
        assert_eq!(None, john.may_load(&store).unwrap());
    }

    #[test]
    fn page_with_accumulator() {

        // Change balance to 0, add balance to total and return value if even
        fn accumulate_and_subtract_balances(key: &[u8], store: &mut dyn Storage, value: Data, acc: &mut IncomeAcc, context: &String) -> StdResult<Option<u32>> {
            let balance = value.balance;
            acc.total += balance;
            USERS.unsafe_save(store, key, &Data{
                balance: 0,
                ..value
            })?;

            if balance%2 == 0 {
                Ok(Some(balance))
            } else {
                Ok(None)
            }
        }

        let mut deps = mock_dependencies(&[]);
        USERS.instantiate(&mut deps.storage).unwrap();
        let mut total = 0;
        let mut even_numbers = vec![];
        
        for i in 0..100 {
            let data = Data {
                name: "IrrelevantName".to_string(),
                balance: i,
            };
            total += data.balance;
            USERS.save(&mut deps.storage, &i.to_be_bytes(), &data).unwrap();
            let stored_data = USERS.load(&deps.storage, &i.to_be_bytes()).unwrap();
            if i%2 == 0 {
                even_numbers.push(i);
            };
            assert_eq!(stored_data, data);
        }
        let mut result_even_numbers = vec![];
        
        // first call, external factor (like a time stamp) should determine when you can start the accumulator.
        let (_, mut maybe_even_numbers) =USERS.page_with_accumulator(deps.as_mut(), None, &String::new(), accumulate_and_subtract_balances).unwrap();
        
        assert!(USERS.status.load(&deps.storage).unwrap().accumulator.is_some());
        assert!(USERS.status.load(&deps.storage).unwrap().is_locked);
        // Keep track of the output
        result_even_numbers.append(&mut maybe_even_numbers);
        
        while USERS.status.load(&deps.storage).unwrap().is_locked {
            let (maybe_accumulator, mut maybe_even_numbers) = USERS.page_with_accumulator(deps.as_mut(), None, &String::new(), accumulate_and_subtract_balances).unwrap();
            
            result_even_numbers.append(&mut maybe_even_numbers);
            
            if let Some(acc) = maybe_accumulator{
                // Accumulator should be done
                assert_eq!(acc.total, total);
                assert_eq!(result_even_numbers, even_numbers);
            }
        }
        for i in 0..100u32 {
            let stored_data = USERS.load(&deps.storage, &i.to_be_bytes()).unwrap();
            assert_eq!(stored_data, Data {
                name: "IrrelevantName".to_string(),
                balance: 0,
            });
        }
    }
}
