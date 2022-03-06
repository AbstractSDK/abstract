use cosmwasm_std::{Deps, DepsMut, Order, StdResult};
use cw_storage_plus::{Bound, Item, Map};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub struct PagedMap<'a, T, R> {
    pub data: Map<'a, &'a [u8], T>,
    pub status: Item<'a, PaginationInfo<R>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationInfo<R> {
    pub is_locked: bool,
    pub counter: u32,
    pub size: u32,
    pub last_processed_item: Option<String>,
    pub accumulator: Option<R>,
}

impl<R> PaginationInfo<R> {
    pub fn progress(&self) -> String {
        format!("Tallied {} of {} ", self.counter, self.size)
    }
}

impl<'a, T, R> PagedMap<'a, T, R> {
    pub const fn new(namespace: &'a str, status_namespace: &'a str) -> Self {
        PagedMap {
            data: Map::new(namespace),
            status: Item::new(status_namespace),
        }
    }

    pub fn page_with_accumulator(
        &self,
        deps: DepsMut,
        limit: Option<u32>,
        f: fn((Vec<u8>, T, Deps), &mut R),
    ) -> StdResult<Option<R>>
    where
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned + Default + Clone,
    {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let mut status = self.status.load(deps.storage)?;
        if status.accumulator.is_none() {
            status.is_locked = true;
            status.accumulator = Some(R::default());
            status.counter = 0u32;
        }
        let start = status.last_processed_item.clone().map(Bound::exclusive);

        let result: Vec<Vec<u8>> = self
            .data
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                let (key, element) = item.unwrap();
                f(
                    (key.clone(), element, deps.as_ref()),
                    &mut status
                        .accumulator
                        .as_mut()
                        .expect("accumulator contains some value"),
                );
                key
            })
            .collect();

        status.counter += result.len() as u32;

        status.last_processed_item = result
            .last()
            .map(|key| String::from(std::str::from_utf8(key).unwrap()));

        let accumulator = PagedMap::<'a, T, R>::is_done(&mut status);

        self.status.save(deps.storage, &status)?;
        Ok(accumulator)
    }

    /// Returns the accumulator if operation is finished
    fn is_done(status: &mut PaginationInfo<R>) -> Option<R>
    where
        R: Clone,
    {
        let accumulator: Option<R>;

        if status.counter == status.size {
            accumulator = status.accumulator.clone();
            status.is_locked = false;
            status.accumulator = None;
        } else {
            accumulator = None;
        }

        accumulator
    }
}
