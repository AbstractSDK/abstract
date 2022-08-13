use std::ops::Mul;

use cosmwasm_std::{Decimal, Env, StdResult, Storage, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PRECISION: u8 = 6;

/// Time Weighted Average (TWA) helper
pub struct TimeWeightedAverage<'a>(Item<'a, TimeWeightedAverageData>);

impl<'a> TimeWeightedAverage<'a> {
    pub const fn new(key: &'a str) -> Self {
        Self(Item::new(key))
    }
    pub fn instantiate(
        &self,
        store: &mut dyn Storage,
        env: &Env,
        precision: Option<u8>,
        averaging_period: u64,
    ) -> StdResult<()> {
        let block_time = env.block.time.seconds();

        let twa = TimeWeightedAverageData {
            cumulative_value: 0,
            last_block_time: block_time,
            precision: precision.unwrap_or(DEFAULT_PRECISION),
            average_value: Decimal::zero(),
            averaging_period,
            last_averaging_cumulative_value: 0,
            last_averaging_block_time: block_time,
            last_averaging_block_height: env.block.height,
        };
        self.0.save(store, &twa)
    }
    pub fn accumulate(
        &self,
        env: &Env,
        store: &mut dyn Storage,
        current_value: Decimal,
    ) -> StdResult<Option<(u128, u64)>> {
        let mut twa = self.0.load(store)?;
        let block_time = env.block.time.seconds();
        if block_time <= twa.last_block_time {
            return Ok(None);
        }

        let time_elapsed = Uint128::from(block_time - twa.last_block_time);
        twa.last_block_time = block_time;

        if !current_value.is_zero() {
            twa.cumulative_value = twa
                .cumulative_value
                .wrapping_add(time_elapsed.mul(current_value).u128());
        };
        self.0.save(store, &twa)?;
        Ok(Some((twa.cumulative_value, block_time)))
    }

    pub fn get_value(&self, store: &dyn Storage) -> StdResult<Decimal> {
        Ok(self.0.load(store)?.average_value)
    }

    pub fn load(&self, store: &dyn Storage) -> StdResult<TimeWeightedAverageData> {
        self.0.load(store)
    }

    /// Get average value, updates when possible
    pub fn try_update_value(
        &self,
        env: &Env,
        store: &mut dyn Storage,
    ) -> StdResult<Option<Decimal>> {
        let mut twa = self.0.load(store)?;

        let block_time = env.block.time.seconds();

        let time_elapsed = block_time - twa.last_averaging_block_time;

        // Ensure that at least one full period has passed since the last update
        if time_elapsed < twa.averaging_period {
            return Ok(None);
        }

        // (current_cum - last_cum) / time
        let new_average_value = Decimal::from_ratio(
            twa.cumulative_value
                .wrapping_sub(twa.last_averaging_cumulative_value),
            time_elapsed,
        );

        twa = TimeWeightedAverageData {
            average_value: new_average_value,
            last_averaging_block_time: block_time,
            last_averaging_cumulative_value: twa.cumulative_value,
            ..twa
        };
        self.0.save(store, &twa)?;
        Ok(Some(new_average_value))
    }

    pub fn update_settings(
        &self,
        _env: &Env,
        store: &mut dyn Storage,
        averaging_period: u64,
    ) -> StdResult<()> {
        let mut twa = self.0.load(store)?;
        twa.averaging_period = averaging_period;
        self.0.save(store, &twa)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TimeWeightedAverageData {
    // settings for accumulating value data
    pub precision: u8,
    pub last_block_time: u64,
    pub cumulative_value: u128,

    // Data to get average price
    pub last_averaging_block_time: u64,
    pub last_averaging_block_height: u64,
    pub last_averaging_cumulative_value: u128,
    pub averaging_period: u64,
    /// The requested average value
    average_value: Decimal,
}
