use cosmwasm_schema::cw_serde;

use cosmwasm_std::{
    Decimal, Decimal256, Env, Fraction, StdError, StdResult, Storage, Timestamp, Uint128, Uint256,
};
use cw_storage_plus::Item;

use crate::asset::{AssetInfo, AssetInfoValidated};

pub const MINUTE: u64 = 60;
pub const HALF_HOUR: u64 = 30 * MINUTE;
pub const SIX_HOURS: u64 = 6 * 60 * MINUTE;

#[cw_serde]
#[derive(Copy)]
pub enum SamplePeriod {
    Minute,
    HalfHour,
    SixHour,
}

const LAST_UPDATES: Item<LastUpdates> = Item::new("oracle_last_updated");
const LAST_MINUTES_PRICES: Item<Prices> = Item::new("oracle_by_minute");
const LAST_HALF_HOUR_PRICES: Item<Prices> = Item::new("oracle_by_half_hour");
const LAST_SIX_HOUR_PRICES: Item<Prices> = Item::new("oracle_by_six_hour");

/// For each price history, stores the last timestamp (in seconds) when it was updated,
/// As well as the last measurement (running accumulator).
/// Snapshot times are measured in full seconds
#[cw_serde]
pub struct LastUpdates {
    pub accumulator: Accumulator,
    pub minutes: u64,
    pub half_hours: u64,
    pub six_hours: u64,
}

/// This is the last snapshot of the price accumulator.
/// This only works on 2 pools:
///   A is config.pair_info.asset_infos[0],
///   B is config.pair_info.asset_infos[1],
/// We keep this pattern and limit how much data is written
#[cw_serde]
pub struct Accumulator {
    /// Last time we measured the accumulator.
    /// Uses nanosecond for subsecond-blocks (eg sei)
    pub snapshot: Timestamp,
    /// Value of a_per_b at that time
    pub last_price: Decimal,
    /// Running accumulator values
    pub twap_a_per_b: Twap,
    pub twap_b_per_a: Twap,
    // FIXME: add this later (https://github.com/wynddao/wyndex-priv/issues/7)
    // pub geometric_a_to_b: Twgm,
}

impl Accumulator {
    pub fn new(now: Timestamp, price: Decimal) -> Self {
        Accumulator {
            snapshot: now,
            last_price: price,
            twap_a_per_b: Default::default(),
            twap_b_per_a: Default::default(),
        }
    }

    /// if env.block.time > self.snapshot, does whole update of twap
    /// if equal, then just updates last_price
    /// if earlier, panics (should never happen)
    pub fn update(&mut self, env: &Env, price: Decimal) {
        use std::cmp::Ordering::*;
        let now = env.block.time;
        match now.cmp(&self.snapshot) {
            Less => {
                panic!("Cannot update from the past");
            }
            Equal => {
                // just update the last price
                self.last_price = price;
            }
            Greater => {
                // we do proper update
                let elapsed = diff_nanos(self.snapshot, now);
                self.twap_a_per_b = self.twap_a_per_b.accumulate_nanos(self.last_price, elapsed);
                // Note: price must never be 0
                self.twap_b_per_a = self
                    .twap_b_per_a
                    .accumulate_nanos(self.last_price.inv().unwrap(), elapsed);
                self.last_price = price;
                self.snapshot = now;
            }
        }
    }
}

pub const BUFFER_DEPTH: usize = 32;

/// This is a buffer of at most [`BUFFER_DEPTH`] size, containing snapshots of the accumulator.
/// Accumulator values are stored newest to oldest. Index 0 is the value as of LastUpdates.<period>
/// Every value after that is the (interpolated) value one <period> after that
/// (eg for the HALF_HOUR range, 3 index steps means 90 minutes)
/// The buffer starts out empty, and is filled in as we go, but always has at most [`BUFFER_DEPTH`] values.
#[cw_serde]
#[derive(Default)]
pub struct Prices {
    pub twap_a_per_b: Vec<Twap>,
    pub twap_b_per_a: Vec<Twap>,
    // FIXME: add this later (https://github.com/wynddao/wyndex-priv/issues/7)
    // pub geometric_a_to_b: [Twgm; BUFFER_DEPTH],
}

impl Prices {
    /// update the whole price buffer, given latest accumulator, last sample time, and current time
    pub fn accumulate(
        &self,
        last_update: u64,
        latest_checkpoint: u64,
        acc: &Accumulator,
        step: u64,
    ) -> Prices {
        let new_checkpoints = ((latest_checkpoint - last_update) / step) as usize;

        let mut new_prices = Prices::default();

        if new_prices.twap_a_per_b.len() < BUFFER_DEPTH {
            // we have not fully filled the buffer yet, so we extend the size first
            // both vectors are the same size, so we only need to calculate this for one of them
            let len = BUFFER_DEPTH.min(self.twap_a_per_b.len() + new_checkpoints);
            new_prices.twap_a_per_b.resize(len, Default::default());
            new_prices.twap_b_per_a.resize(len, Default::default());
        }

        // we copy any still valid ones to their new offset
        // and figure out where we start computing from
        let (last_copied, last_timestamp) = if new_checkpoints < BUFFER_DEPTH {
            let len = new_prices.twap_a_per_b.len();
            new_prices.twap_a_per_b[new_checkpoints..]
                .copy_from_slice(&self.twap_a_per_b[0..len - new_checkpoints]);
            new_prices.twap_b_per_a[new_checkpoints..]
                .copy_from_slice(&self.twap_b_per_a[0..len - new_checkpoints]);
            (new_checkpoints, last_update)
        } else {
            // all are invalid, need to figure out the time that would be at the first one
            let oldest_time = latest_checkpoint - step * ((BUFFER_DEPTH) as u64);
            (BUFFER_DEPTH, oldest_time)
        };

        // * last_timestamp from accumulator
        // * value at that timestamp
        // * last_price from accumulator
        // * time at first index we will be writing to

        for i in 0..last_copied {
            // how much time passed between the accumulator and this checkpoint
            let time = Timestamp::from_seconds(last_timestamp + (last_copied - i) as u64 * step);
            let elapsed_nanos = diff_nanos(acc.snapshot, time);
            // set the new values
            new_prices.twap_a_per_b[i] = acc
                .twap_a_per_b
                .accumulate_nanos(acc.last_price, elapsed_nanos);
            new_prices.twap_b_per_a[i] = acc
                .twap_b_per_a
                .accumulate_nanos(acc.last_price.inv().unwrap(), elapsed_nanos);
        }

        new_prices
    }
}

/// We need more precision than Uint128, but will overflow with Decimal
#[cw_serde]
#[derive(Default, Copy, Eq, PartialOrd, Ord)]
pub struct Twap(Decimal256);

impl Twap {
    /// Give it the time since the last measurement and the value at last snapshot.
    /// It will add (last_price * elapsed_seconds) to the accumulator.
    /// Make sure to be careful with overflow
    #[must_use]
    pub fn accumulate_nanos(&self, last_price: Decimal, elapsed_nanos: u64) -> Twap {
        let numerator = Uint256::from(last_price.numerator()) * Uint256::from(elapsed_nanos);
        // 10^18 from Decimal, 10^9 from nanos
        let increment =
            Decimal256::from_atomics(numerator, Decimal256::DECIMAL_PLACES + 9).unwrap();
        Twap(self.0 + increment)
    }

    #[must_use]
    pub fn accumulate_secs(&self, last_price: Decimal, elapsed_secs: u64) -> Twap {
        let numerator = Uint256::from(last_price.numerator()) * Uint256::from(elapsed_secs);
        let increment = Decimal256::from_atomics(numerator, Decimal256::DECIMAL_PLACES).unwrap();
        Twap(self.0 + increment)
    }

    /// Given two Twap values and the time between them, get the average price in this range
    /// (now - earlier) * 10^9 / elapsed_nanos
    pub fn average_price(&self, earlier: &Twap, elapsed_nanos: u64) -> Decimal {
        let diff = self.0 - earlier.0;
        let atomics = diff.numerator() / Uint256::from(elapsed_nanos);
        Decimal::from_atomics(
            Uint128::try_from(atomics).unwrap(),
            Decimal256::DECIMAL_PLACES - 9,
        )
        .unwrap()
    }
}

/// get the elapsed nanos from older to later
pub fn diff_nanos(older: Timestamp, later: Timestamp) -> u64 {
    later.nanos() - older.nanos()
}

/// This must be called one time when the initial liquidity is added to initialize all the twap counters.
/// It gets the timestamp of the block along with the initial price, and sets up all accumulators
pub fn initialize_oracle(storage: &mut dyn Storage, env: &Env, price: Decimal) -> StdResult<()> {
    let now = env.block.time;

    // save the current value
    let accumulator = Accumulator::new(now, price);
    let last_updates = LastUpdates {
        accumulator,
        minutes: now.seconds(),
        half_hours: now.seconds(),
        six_hours: now.seconds(),
    };
    LAST_UPDATES.save(storage, &last_updates)?;

    // set empty prices (0 for all accumulators)
    let empty_prices = Prices::default();
    LAST_MINUTES_PRICES.save(storage, &empty_prices)?;
    LAST_HALF_HOUR_PRICES.save(storage, &empty_prices)?;
    LAST_SIX_HOUR_PRICES.save(storage, &empty_prices)?;

    Ok(())
}

/// This is called every time the price changes in the pool.
/// If this is the same timestamp as the last update (same block), we just update last_price
/// If it is later timestamp, we update the accumulator, and possibly update historical values
pub fn store_oracle_price(
    storage: &mut dyn Storage,
    env: &Env,
    new_price_a_per_b: Decimal,
) -> StdResult<()> {
    let mut updates = LAST_UPDATES.load(storage)?;
    // if the block time is exactly the minute timestamp, we already updated within this block, just track last_price
    if env.block.time == updates.accumulator.snapshot {
        updates.accumulator.last_price = new_price_a_per_b;
        LAST_UPDATES.save(storage, &updates)?;
        return Ok(());
    }

    // update if full minute has passed since last time
    if let Some(latest_checkpoint) = calc_checkpoint(updates.minutes, env, MINUTE) {
        let old_prices = LAST_MINUTES_PRICES.load(storage)?;
        let prices = old_prices.accumulate(
            updates.minutes,
            latest_checkpoint,
            &updates.accumulator,
            MINUTE,
        );
        updates.minutes = latest_checkpoint;
        LAST_MINUTES_PRICES.save(storage, &prices)?;
    }

    // update if full half hour has passed since last time
    if let Some(latest_checkpoint) = calc_checkpoint(updates.half_hours, env, HALF_HOUR) {
        let old_prices = LAST_HALF_HOUR_PRICES.load(storage)?;
        let prices = old_prices.accumulate(
            updates.half_hours,
            latest_checkpoint,
            &updates.accumulator,
            HALF_HOUR,
        );
        updates.half_hours = latest_checkpoint;
        LAST_HALF_HOUR_PRICES.save(storage, &prices)?;
    }

    // update if full six hour has passed since last time
    if let Some(latest_checkpoint) = calc_checkpoint(updates.six_hours, env, SIX_HOURS) {
        let old_prices = LAST_SIX_HOUR_PRICES.load(storage)?;
        let prices = old_prices.accumulate(
            updates.six_hours,
            latest_checkpoint,
            &updates.accumulator,
            SIX_HOURS,
        );
        updates.six_hours = latest_checkpoint;
        LAST_SIX_HOUR_PRICES.save(storage, &prices)?;
    }

    // always update the current accumulator (after calculations are finished to not interfere)
    updates.accumulator.update(env, new_price_a_per_b);
    LAST_UPDATES.save(storage, &updates)
}

/// This finds the most recent checkpoint before the current moment.
/// Returns None if there is nothing more recent than the latest update.
fn calc_checkpoint(last_update: u64, env: &Env, step: u64) -> Option<u64> {
    let steps = (env.block.time.seconds() - last_update) / step;
    if steps == 0 {
        None
    } else {
        Some(last_update + steps * step)
    }
}

#[cw_serde]
pub struct TwapResponse {
    pub a: AssetInfo,
    pub b: AssetInfo,
    pub a_per_b: Decimal,
    pub b_per_a: Decimal,
}

/// This gets the twap for a range, which must be one of our sample frequencies, within the depth we maintain
pub fn query_oracle_range(
    storage: &dyn Storage,
    env: &Env,
    asset_infos: &[AssetInfoValidated],
    // This is the resolution of the buffer we wish to read
    sample_period: SamplePeriod,
    // This is the beginning of the period, measured in how many full samples back we start
    // 4 would start 4 full sample periods earlier than the end of the time buffer
    start_index: u32,
    // This is the end of the period, measured in how many full samples back we end.
    // Some(0) takes the last item on the stored buffer.
    // None takes the latest accumulator update
    end_index: Option<u32>,
) -> StdResult<TwapResponse> {
    // TODO: assert start_index > end_index

    let updates = LAST_UPDATES.load(storage)?;
    let (step, last_update, stored_prices) = match sample_period {
        SamplePeriod::Minute => (MINUTE, updates.minutes, LAST_MINUTES_PRICES.load(storage)?),
        SamplePeriod::HalfHour => (
            HALF_HOUR,
            updates.half_hours,
            LAST_HALF_HOUR_PRICES.load(storage)?,
        ),
        SamplePeriod::SixHour => (
            SIX_HOURS,
            updates.six_hours,
            LAST_SIX_HOUR_PRICES.load(storage)?,
        ),
    };

    // interpolate prices to the present (if they haven't been updated in a while)
    let latest_checkpoint = calc_checkpoint(last_update, env, step);
    let (_checkpoint, prices) = match latest_checkpoint {
        Some(checkpoint) => (
            checkpoint,
            stored_prices.accumulate(last_update, checkpoint, &updates.accumulator, step),
        ),
        None => (last_update, stored_prices),
    };

    let old_twap_a_per_b = prices
        .twap_a_per_b
        .get(start_index as usize)
        .ok_or_else(|| {
            StdError::generic_err("start index is earlier than earliest recorded price data")
        })?;
    let old_twap_b_per_a = prices.twap_b_per_a[start_index as usize];

    // handle current accumulator (`end_index == None`)
    let (elapsed_nanos, new_twap_a_per_b, new_twap_b_per_a) = match end_index {
        Some(end_index) => {
            let elapsed_nanos = step * 1_000_000_000u64 * (start_index - end_index) as u64;

            (
                elapsed_nanos,
                prices.twap_a_per_b[end_index as usize],
                prices.twap_b_per_a[end_index as usize],
            )
        }
        None => {
            // in this case, we calculate time between start entry and the last buffer entry and add the time since the last update
            let elapsed_nanos = step * 1_000_000_000u64 * start_index as u64
                + env.block.time.nanos()
                - last_update * 1_000_000_000u64;

            let elapsed_since_acc = env.block.time.nanos() - updates.accumulator.snapshot.nanos();
            (
                elapsed_nanos,
                updates
                    .accumulator
                    .twap_a_per_b
                    .accumulate_nanos(updates.accumulator.last_price, elapsed_since_acc),
                updates.accumulator.twap_b_per_a.accumulate_nanos(
                    updates.accumulator.last_price.inv().unwrap(),
                    elapsed_since_acc,
                ),
            )
        }
    };

    let a_per_b = new_twap_a_per_b.average_price(old_twap_a_per_b, elapsed_nanos);
    let b_per_a = new_twap_b_per_a.average_price(&old_twap_b_per_a, elapsed_nanos);

    Ok(TwapResponse {
        a: asset_infos[0].clone().into(),
        b: asset_infos[1].clone().into(),
        a_per_b,
        b_per_a,
    })
}

/// This gets the twap for a range, which must be one of our sample frequencies, within the depth we maintain
pub fn query_oracle_accumulator(storage: &dyn Storage) -> StdResult<Accumulator> {
    Ok(LAST_UPDATES.load(storage)?.accumulator)
}

#[cfg(test)]
mod tests {
    use crate::oracle::{Accumulator, Twap, BUFFER_DEPTH};
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::{assert_approx_eq, Decimal, Fraction, Timestamp, Uint128};

    use super::{calc_checkpoint, Prices, MINUTE};

    #[test]
    fn twap_accumulates() {
        // Test 10 s at 3, 10s at 2, 10s at 1... see average
        let orig = Twap::default();
        let first = orig.accumulate_nanos(Decimal::percent(300), 10_000_000_000);
        let second = first.accumulate_nanos(Decimal::percent(200), 10_000_000_000);
        let third = second.accumulate_nanos(Decimal::percent(100), 10_000_000_000);

        // find averages over all time
        let total_avg = third.average_price(&orig, 30_000_000_000);
        assert_eq!(total_avg, Decimal::percent(200));

        // this has 10s at 2, 10s at 1
        let partial_avg = third.average_price(&first, 20_000_000_000);
        assert_eq!(partial_avg, Decimal::percent(150));
    }

    #[test]
    fn updating_accumulator() {
        let step = 15u64;
        let time = Timestamp::from_seconds(1682155831);

        // this is history that will be ignored
        let mut acc = Accumulator::new(time, Decimal::percent(1700));

        // for the start of our counting era, the price is 3
        let mut env = mock_env();
        let time = time.plus_seconds(500);
        env.block.time = time;
        acc.update(&env, Decimal::percent(300));
        let orig = acc.clone();

        // after one "step", drops down to 1.00
        env.block.time = time.plus_seconds(step);
        acc.update(&env, Decimal::percent(100));

        // after another step, comes up to 2.00
        env.block.time = time.plus_seconds(step * 2);
        acc.update(&env, Decimal::percent(200));

        // after another step moves to 5 (doesn't matter as this time is not included)
        env.block.time = time.plus_seconds(step * 3);
        acc.update(&env, Decimal::percent(500));

        // ensure other attributes set
        assert_eq!(acc.last_price, Decimal::percent(500));
        assert_eq!(acc.snapshot, env.block.time);

        // average a_per_b price should be (3 + 1 + 2) / 3 = 2
        let a_per_b = acc
            .twap_a_per_b
            .average_price(&orig.twap_a_per_b, step * 3 * 1_000_000_000);
        assert_eq!(a_per_b, Decimal::percent(200));

        // average b_per_a price should be (1/3 + 1 + 1/2) / 3 = 11/18
        let b_per_a = acc
            .twap_b_per_a
            .average_price(&orig.twap_b_per_a, step * 3 * 1_000_000_000);
        let expected = Decimal::from_ratio(11u128, 18u128);
        // they should be close to 1 part per 1_000_000 (rounding)
        assert_eq!(
            b_per_a * Uint128::new(1_000_000),
            expected * Uint128::new(1_000_000)
        );
    }

    #[test]
    fn updating_price_buffer() {
        let mut prices = Prices::default();

        let mut env = mock_env();
        // set price at 2.0
        let accumulator: Accumulator =
            Accumulator::new(env.block.time, Decimal::from_atomics(2u128, 0).unwrap());
        let last_update = env.block.time.seconds();

        // wait two minutes and accumulate
        env.block.time = env.block.time.plus_seconds(120);
        let checkpoint = calc_checkpoint(last_update, &env, MINUTE).unwrap();
        prices = prices.accumulate(last_update, checkpoint, &accumulator, MINUTE);

        // query the twap price at 1 minute ago vs now (should be 2.0)
        let old_twap = prices.twap_a_per_b[1];
        let new_twap = prices.twap_a_per_b[0];
        let a_per_b = new_twap.average_price(&old_twap, MINUTE * 1_000_000_000u64);

        assert_approx_eq!(
            a_per_b.numerator(),
            Decimal::from_atomics(2u128, 0).unwrap().numerator(),
            "0.00002"
        );
    }

    #[test]
    fn long_gap_between_prices() {
        let mut prices = Prices::default();

        let mut env = mock_env();
        // set price at 2.0
        let accumulator = Accumulator::new(env.block.time, Decimal::percent(200));
        let last_update = env.block.time.seconds();

        // wait 10.5 minutes and accumulate
        env.block.time = env.block.time.plus_seconds(10 * 60 + 30);
        let checkpoint = calc_checkpoint(last_update, &env, MINUTE).unwrap();
        prices = prices.accumulate(last_update, checkpoint, &accumulator, MINUTE);

        let new_twap = prices.twap_a_per_b[0];
        for i in 1..=9 {
            // check `i` minutes ago vs latest
            let old_twap = prices.twap_a_per_b[i];

            let a_per_b = new_twap.average_price(&old_twap, i as u64 * MINUTE * 1_000_000_000u64);
            assert_eq!(a_per_b.numerator(), Decimal::percent(200).numerator());
        }
        assert_eq!(prices.twap_a_per_b.len(), 10);
    }

    #[test]
    fn buffer_len() {
        let mut prices = Prices::default();

        let mut env = mock_env();
        let mut accumulator = Accumulator::new(env.block.time, Decimal::one());
        let last_update = env.block.time.seconds();

        // wait 1 second and accumulate
        env.block.time = env.block.time.plus_seconds(1);
        let checkpoint = calc_checkpoint(last_update, &env, 1).unwrap();
        prices = prices.accumulate(last_update, checkpoint, &accumulator, 1);
        let last_update = env.block.time.seconds();
        // change accumulator price
        accumulator.update(&env, Decimal::percent(200));

        // wait `BUFFER_DEPTH` seconds and accumulate (this should overwrite the first entry)
        env.block.time = env.block.time.plus_seconds(BUFFER_DEPTH as u64);
        let checkpoint = calc_checkpoint(last_update, &env, 1).unwrap();
        assert_eq!(checkpoint, last_update + BUFFER_DEPTH as u64);
        prices = prices.accumulate(last_update, checkpoint, &accumulator, 1);

        // all TWAPs should come out to the new price, since the first entry was overwritten
        let latest = prices.twap_a_per_b[0];
        for i in 1..BUFFER_DEPTH {
            assert_eq!(
                latest.average_price(&prices.twap_a_per_b[i], i as u64 * 1_000_000_000u64),
                Decimal::percent(200)
            );
        }

        assert_eq!(prices.twap_a_per_b.len(), BUFFER_DEPTH);
    }
}
