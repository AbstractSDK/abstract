use cosmwasm_std::{CosmosMsg, Deps, DepsMut, Empty, Order, StdError, StdResult, Storage};
use cw_storage_plus::{Bound, Item, Map};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::deposit_manager::Deposit;
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;
const MAX_MSG_LIMIT: u32 = 15;

pub type PaginationResult<A, PageResult> = StdResult<(Option<A>,PageResult)>;
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

    pub fn load(&self, store: & dyn Storage, key: &[u8]) -> StdResult<T>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.load(store, key)
    }

    pub fn has(&self, store: & dyn Storage, key: &[u8]) -> bool
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.has(store, key)
    }

    pub fn may_load(&self, store: & dyn Storage, key: &[u8]) -> StdResult<Option<T>>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.data.may_load(store, key)
    }

    pub fn load_status(&self, store: & dyn Storage) -> StdResult<PaginationInfo<R>>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        self.status.load(store)
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
        f: fn(&[u8],&mut dyn Storage, T, &mut R, &C) -> StdResult<Option<FuncResult>>,
    ) -> PaginationResult<R,Vec<FuncResult>>
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
        let accumulator = if !result.is_empty() {
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
        
        
        let function_results = result.into_iter().filter_map(|(key, element)| {
            f(
                &key,
                    deps.storage,
                    element,
                    status
                        .accumulator
                        .as_mut()
                        .expect("accumulator contains some value"),
                    context
                ).ok().unwrap_or(None)
            }).collect::<Vec<FuncResult>>();
            
        self.status.save(deps.storage, &status)?;

        Ok((accumulator,function_results))
    }

    /// Will apply function on each element (key, value) of the map. Errors on function f() are neglected. 
    pub fn page_for_msgs<C>(
        &self,
        deps: DepsMut,
        limit: Option<u32>,
        context: &C,
        f: fn((Vec<u8>, &mut T), &mut dyn Storage, context: &C) -> StdResult<Option<CosmosMsg<Empty>>>,
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
            status.last_processed_item = result
                .last()
                .map(|(key, _)| key.clone());
            Some(
                result
                    .into_iter()
                    .flat_map(|(key, mut item)| f((key,&mut item), deps.storage, context).ok()).flatten()
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
