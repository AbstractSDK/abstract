use cosmwasm_std::{DepsMut, Order, StdError, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map, Path};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;
const MAX_MSG_LIMIT: u32 = 15;

pub type PaginationResult<Acum, PageResult> = StdResult<(Option<Acum>, PageResult)>;
pub type PaginationAccumulatorFunction<T, Acum, C, FuncResult> =
    fn(&[u8], &mut dyn Storage, T, &mut Acum, &C) -> StdResult<Option<FuncResult>>;
pub type PaginationFunction<T, C, FuncResult> =
    fn(&[u8], &mut dyn Storage, T, &C) -> StdResult<Option<FuncResult>>;
/// Allows for multi-transaction computation on a dataset. Required for large datasets due to gas constraints.
pub struct PagedMap<'a, T, Acum> {
    /// Actual data store
    data: Map<'a, &'a [u8], T>,
    /// Pagination progress status
    pub status: Item<'a, PaginationInfo<Acum>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationInfo<Acum> {
    /// Prevents map manipulation during pagination
    pub is_locked: bool,
    /// Starting item for next iteration
    pub last_processed_item: Option<Vec<u8>>,
    /// Accumulator item available for use in pagination function
    pub accumulator: Option<Acum>,
}

impl<'a, T, Acum> PagedMap<'a, T, Acum> {
    pub const fn new(namespace: &'a str, status_namespace: &'a str) -> Self {
        PagedMap {
            data: Map::new(namespace),
            status: Item::new(status_namespace),
        }
    }

    pub fn instantiate(&self, store: &mut dyn Storage) -> Result<(), StdError>
    where
        T: Serialize + DeserializeOwned,
        Acum: Serialize + DeserializeOwned + Default + Clone,
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
        Acum: Serialize + DeserializeOwned + Default + Clone,
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
        Acum: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.save(store, key, data)
    }

    // Returns the removed item after deleting it
    pub fn remove(&self, store: &mut dyn Storage, key: &[u8]) -> StdResult<T>
    where
        T: Serialize + DeserializeOwned,
        Acum: Serialize + DeserializeOwned + Default + Clone,
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
        Acum: Serialize + DeserializeOwned + Default + Clone,
    {
        let old_item = self.data.load(store, key)?;
        self.data.remove(store, key);

        Ok(old_item)
    }

    pub fn load(&self, store: &dyn Storage, key: &[u8]) -> StdResult<T>
    where
        T: Serialize + DeserializeOwned,
        Acum: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.load(store, key)
    }

    pub fn has(&self, store: &dyn Storage, key: &[u8]) -> bool
    where
        T: Serialize + DeserializeOwned,
        Acum: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.has(store, key)
    }

    pub fn may_load(&self, store: &dyn Storage, key: &[u8]) -> StdResult<Option<T>>
    where
        T: Serialize + DeserializeOwned,
        Acum: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.may_load(store, key)
    }

    pub fn load_status(&self, store: &dyn Storage) -> StdResult<PaginationInfo<Acum>>
    where
        T: Serialize + DeserializeOwned,
        Acum: Serialize + DeserializeOwned + Default + Clone,
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
        f: PaginationAccumulatorFunction<T, Acum, C, FuncResult>,
    ) -> PaginationResult<Acum, Vec<FuncResult>>
    where
        T: Serialize + DeserializeOwned,
        Acum: Serialize + DeserializeOwned + Default + Clone,
    {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let mut status = self.status.load(deps.storage)?;
        if !status.is_locked {
            status.is_locked = true;
            status.accumulator = Some(Acum::default());
            status.last_processed_item = None;
        }

        let start = status.last_processed_item.clone().map(Bound::ExclusiveRaw);

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
            let accumulator: Option<Acum> = status.accumulator.clone();
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
    /// Will not lock the set as no accumulator is used so map state changes are allowed.
    pub fn page_without_accumulator<C, FuncResult>(
        &self,
        deps: DepsMut,
        limit: Option<u32>,
        context: &C,
        f: PaginationFunction<T, C, FuncResult>,
    ) -> StdResult<Vec<FuncResult>>
    where
        T: Serialize + DeserializeOwned,
        Acum: Serialize + DeserializeOwned + Default + Clone,
    {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_MSG_LIMIT) as usize;
        let mut status = self.status.load(deps.storage)?;

        let start = status.last_processed_item.clone().map(Bound::ExclusiveRaw);
        let result: Vec<(Vec<u8>, T)> = self
            .data
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .collect::<StdResult<Vec<(Vec<u8>, T)>>>()?;

        // If not all items processed, update last item
        if !result.is_empty() {
            let last_key = result.last().unwrap().0.clone();
            status.last_processed_item = Some(last_key);
        } else {
            // Everything processed, unlock map
            status.last_processed_item = None;
        };

        let function_results = result
            .into_iter()
            .filter_map(|(key, element)| {
                f(&key, deps.storage, element, context).ok().unwrap_or(None)
            })
            .collect::<Vec<FuncResult>>();

        self.status.save(deps.storage, &status)?;

        Ok(function_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    use cosmwasm_std::testing::{mock_dependencies, MockStorage};

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
    struct Data {
        pub name: String,
        pub balance: u32,
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
    struct IncomeAcc {
        pub total: u32,
    }

    const USERS: PagedMap<Data, IncomeAcc> = PagedMap::new("people", "status");

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
        fn accumulate_and_subtract_balances(
            key: &[u8],
            store: &mut dyn Storage,
            mut value: Data,
            acc: &mut IncomeAcc,
            _context: &String,
        ) -> StdResult<Option<u32>> {
            let balance = value.balance;
            value.balance = 0;
            acc.total += balance;
            USERS.unsafe_save(store, key, &value)?;

            if balance % 2 == 0 {
                Ok(Some(balance))
            } else {
                Ok(None)
            }
        }

        let mut deps = mock_dependencies();
        USERS.instantiate(&mut deps.storage).unwrap();
        let mut total = 0;
        let mut even_numbers = vec![];

        for i in 0..100 {
            let data = Data {
                name: "IrrelevantName".to_string(),
                balance: i,
            };
            total += data.balance;
            USERS
                .save(&mut deps.storage, &i.to_be_bytes(), &data)
                .unwrap();
            let stored_data = USERS.load(&deps.storage, &i.to_be_bytes()).unwrap();
            if i % 2 == 0 {
                even_numbers.push(i);
            };
            assert_eq!(stored_data, data);
        }
        let mut result_even_numbers = vec![];

        // first call, external factor (like a time stamp) should determine when you can start the accumulator.
        let (_, mut maybe_even_numbers) = USERS
            .page_with_accumulator(
                deps.as_mut(),
                None,
                &String::new(),
                accumulate_and_subtract_balances,
            )
            .unwrap();

        assert!(USERS
            .status
            .load(&deps.storage)
            .unwrap()
            .accumulator
            .is_some());
        assert!(USERS.status.load(&deps.storage).unwrap().is_locked);
        // Keep track of the output
        result_even_numbers.append(&mut maybe_even_numbers);

        while USERS.status.load(&deps.storage).unwrap().is_locked {
            let (maybe_accumulator, mut maybe_even_numbers) = USERS
                .page_with_accumulator(
                    deps.as_mut(),
                    None,
                    &String::new(),
                    accumulate_and_subtract_balances,
                )
                .unwrap();

            result_even_numbers.append(&mut maybe_even_numbers);

            if let Some(acc) = maybe_accumulator {
                // Accumulator should be done
                assert_eq!(acc.total, total);
                assert_eq!(result_even_numbers, even_numbers);
            }
        }
        for i in 0..100u32 {
            let stored_data = USERS.load(&deps.storage, &i.to_be_bytes()).unwrap();
            assert_eq!(
                stored_data,
                Data {
                    name: "IrrelevantName".to_string(),
                    balance: 0,
                }
            );
        }
    }

    #[test]
    fn page_without_accumulator() {
        // Change balance to 0, add balance to total and return value if even
        fn subtract_balances(
            key: &[u8],
            store: &mut dyn Storage,
            mut value: Data,
            _context: &String,
        ) -> StdResult<Option<u32>> {
            let balance = value.balance;
            value.balance = 0;
            USERS.unsafe_save(store, key, &value)?;

            if balance % 2 == 0 {
                Ok(Some(balance))
            } else {
                Ok(None)
            }
        }

        let mut deps = mock_dependencies();
        USERS.instantiate(&mut deps.storage).unwrap();
        let mut even_numbers = vec![];

        for i in 0..100 {
            let data = Data {
                name: "IrrelevantName".to_string(),
                balance: i,
            };
            USERS
                .save(&mut deps.storage, &i.to_be_bytes(), &data)
                .unwrap();
            let stored_data = USERS.load(&deps.storage, &i.to_be_bytes()).unwrap();
            if i % 2 == 0 {
                even_numbers.push(i);
            };
            assert_eq!(stored_data, data);
        }
        let mut result_even_numbers = vec![];

        // first call, external factor (like a time stamp) should determine when you can start the accumulator.
        let mut maybe_even_numbers = USERS
            .page_without_accumulator(deps.as_mut(), None, &String::new(), subtract_balances)
            .unwrap();

        assert!(!USERS.status.load(&deps.storage).unwrap().is_locked);
        // Keep track of the output
        result_even_numbers.append(&mut maybe_even_numbers);

        while USERS
            .status
            .load(&deps.storage)
            .unwrap()
            .last_processed_item
            .is_some()
        {
            let mut maybe_even_numbers = USERS
                .page_without_accumulator(deps.as_mut(), None, &String::new(), subtract_balances)
                .unwrap();

            result_even_numbers.append(&mut maybe_even_numbers);
        }

        assert_eq!(result_even_numbers, even_numbers);

        for i in 0..100u32 {
            let stored_data = USERS.load(&deps.storage, &i.to_be_bytes()).unwrap();
            assert_eq!(
                stored_data,
                Data {
                    name: "IrrelevantName".to_string(),
                    balance: 0,
                }
            );
        }
    }
}
