use std::collections::HashMap;

use cosmwasm_schema::{
    cw_serde,
    serde::{de::DeserializeOwned, Serialize},
};
use cosmwasm_std::{Deps, Env, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};

use crate::{
    asset::AssetInfoValidated,
    pair::{ContractError, HistoricalPricesResponse, HistoryDuration},
};

const MINUTE: u64 = 60;
const HALF_HOUR: u64 = 30 * MINUTE;
const TWELVE_HOURS: u64 = 60 * MINUTE;

/// For each price history, stores the last timestamp (in seconds) when it was updated
#[cw_serde]
#[derive(Default)]
struct LastUpdates {
    pub minutes: u64,
    pub half_hours: u64,
    pub twelve_hours: u64,
}

const LAST_UPDATED: Item<LastUpdates> = Item::new("oracle_last_updated");
const LAST_15MINUTES_PRICES: TimeBuffer<Vec<Uint128>, MINUTE, 15> =
    TimeBuffer::new("oracle_last_15minutes", "oracle_last_15minutes_first");
const LAST_DAY_PRICES: TimeBuffer<Vec<Uint128>, HALF_HOUR, 48> =
    TimeBuffer::new("oracle_last_day", "oracle_last_day_first");
const LAST_WEEK_PRICES: TimeBuffer<Vec<Uint128>, TWELVE_HOURS, 14> =
    TimeBuffer::new("oracle_last_week", "oracle_last_week_first");

pub struct PricePoint {
    /// the asset that is being swapped from
    pub from: AssetInfoValidated, // TODO: borrow to avoid clone?
    /// the asset that is being swapped to
    pub to: AssetInfoValidated,
    /// the price of the swap
    pub price: Uint128,
}

impl PricePoint {
    pub fn new(from: AssetInfoValidated, to: AssetInfoValidated, price: Uint128) -> Self {
        Self { from, to, price }
    }
}

/// Stores the price of the asset for TWAP calculations and
pub fn store_price(
    storage: &mut dyn Storage,
    env: &Env,
    asset_infos: &[AssetInfoValidated],
    mut prices: Vec<PricePoint>,
) -> Result<(), ContractError> {
    let mut last_updated = LAST_UPDATED.may_load(storage)?.unwrap_or_default();
    if env.block.time.seconds() == last_updated.minutes {
        // if the block time is exactly the minute timestamp, we already updated within this block,
        // no need to update
        return Ok(());
    }

    // get the correct order of the price entries
    let asset_info_index = cartesian_product(asset_infos)
        .map(|(a, b)| (a, b))
        .enumerate()
        .map(|(i, a)| (a, i))
        .collect::<HashMap<_, _>>();
    prices.sort_by(|a, b| {
        asset_info_index[&(&a.from, &a.to)].cmp(&asset_info_index[&(&b.from, &b.to)])
    });
    let prices: Vec<_> = prices.into_iter().map(|p| p.price).collect();

    let mut updated = false;

    if last_updated.minutes + MINUTE <= env.block.time.seconds() {
        // update every minute
        LAST_15MINUTES_PRICES.save(storage, env.block.time.seconds(), prices.clone())?;
        last_updated.minutes = env.block.time.seconds();
        updated = true;
    }

    if last_updated.half_hours + HALF_HOUR <= env.block.time.seconds() {
        // update every half hour
        LAST_DAY_PRICES.save(storage, env.block.time.seconds(), prices.clone())?;
        last_updated.half_hours = env.block.time.seconds();
        updated = true;
    }

    if last_updated.twelve_hours + TWELVE_HOURS <= env.block.time.seconds() {
        // update every 12 hours
        LAST_WEEK_PRICES.save(storage, env.block.time.seconds(), prices)?;
        last_updated.twelve_hours = env.block.time.seconds();
        updated = true;
    }

    if updated {
        LAST_UPDATED.save(storage, &last_updated)?;
    }
    Ok(())
}

pub fn query_historical(
    deps: Deps,
    env: &Env,
    asset_infos: Vec<AssetInfoValidated>,
    duration: HistoryDuration,
) -> StdResult<HistoricalPricesResponse> {
    match duration {
        HistoryDuration::FifteenMinutes => {
            query_timebuffer(deps, env, &LAST_15MINUTES_PRICES, asset_infos)
        }
        HistoryDuration::Day => query_timebuffer(deps, env, &LAST_DAY_PRICES, asset_infos),
        HistoryDuration::Week => query_timebuffer(deps, env, &LAST_WEEK_PRICES, asset_infos),
    }
}

/// Returns the last day of price history for each asset combination.
/// Make sure the `asset_infos` are ordered correctly.
fn query_timebuffer<const STEP: u64, const CAP: u64>(
    deps: Deps,
    env: &Env,
    buffer: &TimeBuffer<Vec<Uint128>, STEP, CAP>,
    asset_infos: Vec<AssetInfoValidated>,
) -> StdResult<HistoricalPricesResponse> {
    // buffer.all returns a vec with all prices for each timestamp,
    // but we want to return one vec with all prices *per asset combination*
    let combinations = combinations(&asset_infos);

    let mut cumulative_prices: Vec<_> = combinations
        .into_iter()
        .map(|(from, to)| (from, to, vec![]))
        .collect();

    for result in buffer.all(deps.storage, env)? {
        let (timestamp, prices) = result?;
        for i in 0..cumulative_prices.len() {
            cumulative_prices[i].2.push((timestamp, prices[i]));
        }
    }

    Ok(HistoricalPricesResponse {
        historical_prices: cumulative_prices,
    })
}

fn combinations(
    asset_infos: &[AssetInfoValidated],
) -> Vec<(AssetInfoValidated, AssetInfoValidated)> {
    let mut combinations =
        Vec::with_capacity(asset_infos.len() * asset_infos.len() - asset_infos.len());
    for from in asset_infos {
        for to in asset_infos {
            if from != to {
                combinations.push((from.clone(), to.clone()));
            }
        }
    }
    combinations
}

fn cartesian_product(
    asset_infos: &[AssetInfoValidated],
) -> impl Iterator<Item = (&AssetInfoValidated, &AssetInfoValidated)> {
    asset_infos.iter().flat_map(move |from| {
        asset_infos
            .iter()
            .filter(|to| !from.equal(to))
            .map(move |to| (from, to))
    })
}

/// A ringbuffer, intended for using timestamps as indices.
///
/// # Layout
/// The buffer is stored in a `Map` with an index (derived from the timestamp) as key and both timestamp and `T` as value.
/// It also contains a metadata item that stores the index of the first element and its timestamp.
/// The map does not give a lot of guarantees, except that all valid entries are ordered by their timestamp.
/// There are, however, possibly invalid entries stored. These are entries that are older than the start time.
/// These are not removed, but filtered out when loading the buffer.
struct TimeBuffer<'a, T: Serialize + DeserializeOwned, const STEP: u64, const CAP: u64> {
    /// The actual data. The value contains both the timestamp and the value.
    data: Map<'a, u64, (u64, T)>,
    /// The start index. This is subtracted from all indices to get the actual index.
    /// It will cycle through the `data` index space.
    start_idx: Item<'a, StartData>,
}

#[cw_serde]
struct StartData {
    /// The index inside the map that corresponds to the start time.
    index: u64,
    /// The timestamp when the buffer starts. All indices are calculated relative to this.
    time: u64,
}

impl<'a, T: Serialize + DeserializeOwned + std::fmt::Debug, const STEP: u64, const CAP: u64>
    TimeBuffer<'a, T, STEP, CAP>
{
    pub const fn new(name: &'a str, start_idx: &'a str) -> Self {
        Self {
            data: Map::new(name),
            start_idx: Item::new(start_idx),
        }
    }

    /// Save the given timestamp and value to the buffer.
    /// Note: Make sure to only save timestamps in increasing order
    /// (or more specifically: Do not store a value chronologically before the oldest entry).
    pub fn save(&self, storage: &mut dyn Storage, timestamp: u64, value: T) -> StdResult<()> {
        // get start index, defaulting to the given timestamp if not set
        let mut save_start = false;
        let mut start = match self.start_idx.may_load(storage)? {
            Some(start) => start,
            None => {
                let start = StartData {
                    index: 0,
                    time: timestamp,
                };
                save_start = true;
                start
            }
        };

        if timestamp < start.time {
            panic!(
                "Only limited timetravel is supported. Cannot store data before the first entry."
            );
        }

        // calculate the index inside the buffer
        let actual_idx = (timestamp - start.time) / STEP + start.index;
        if timestamp >= start.time + STEP * CAP {
            // we skipped over the start index, so we need to move the start index behind the latest entry
            start.index = (actual_idx + 1) % CAP;
            // also update the timestamp of the start index
            // it should be CAP steps behind the latest entry
            start.time += (actual_idx + 1 - CAP) * STEP;
            save_start = true;
        }
        if save_start {
            self.start_idx.save(storage, &start)?;
        }
        // wrap index to the buffer capacity
        let actual_idx = actual_idx % CAP;

        self.data.save(storage, actual_idx, &(timestamp, value))?;
        Ok(())
    }

    pub fn all<'b>(
        &self,
        storage: &'a dyn Storage,
        env: &Env,
    ) -> StdResult<impl Iterator<Item = StdResult<(u64, T)>> + 'b>
    where
        T: 'b,
        'a: 'b,
    {
        let start_data = self.start_idx.may_load(storage)?.unwrap_or(StartData {
            index: 0,
            time: env.block.time.seconds(),
        });

        Ok(self
            .data
            // range from start index to end of `data`
            .range(
                storage,
                Some(Bound::inclusive(start_data.index)),
                None,
                Order::Ascending,
            )
            // then from start of `data` to start index
            .chain(self.data.range(
                storage,
                None,
                Some(Bound::exclusive(start_data.index)),
                Order::Ascending,
            ))
            .filter(move |res| Self::is_valid(start_data.time, res))
            .map(|res| res.map(|(_, (timestamp, value))| (timestamp, value))))
    }

    fn is_valid(start_time: u64, res: &StdResult<(u64, (u64, T))>) -> bool {
        // keep only errors and valid entries
        match res {
            Ok((_, (time, _))) => *time >= start_time,
            Err(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_env, MockStorage};

    #[test]
    fn timebuffer() {
        let mut storage = MockStorage::default();
        let env = mock_env();

        // buffer that holds 10 entries, with a 1 second step in between
        let buffer = TimeBuffer::<u64, 1, 10>::new("test", "test_start");

        // empty buffer
        let entries = buffer
            .all(&storage, &env)
            .unwrap()
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(entries, vec![]);

        let now = env.block.time.seconds();

        buffer.save(&mut storage, now, 1).unwrap();
        buffer.save(&mut storage, now + 1, 2).unwrap();
        // leave `now + 2` empty
        buffer.save(&mut storage, now + 3, 4).unwrap();

        let entries = buffer
            .all(&storage, &env)
            .unwrap()
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(entries, vec![(now, 1), (now + 1, 2), (now + 3, 4)]);

        // wrap around, overwriting `now`
        buffer.save(&mut storage, now + 10, 5).unwrap();

        let entries = buffer
            .all(&storage, &env)
            .unwrap()
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(entries, vec![(now + 1, 2), (now + 3, 4), (now + 10, 5)]);

        // wrap around multiple times, overwriting all other entries
        buffer.save(&mut storage, now + 35, 6).unwrap();

        let entries = buffer
            .all(&storage, &env)
            .unwrap()
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(entries, vec![(now + 35, 6)]);
    }
}
