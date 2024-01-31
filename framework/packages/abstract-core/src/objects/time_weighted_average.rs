//! # Time Weighted Average (TWA) helper
//!
//! A time weighted average is an accumulating value that is updated irregularly.
//! Whenever an update is applied, the time between the current update and the last update is used, along with the current value,
//! to accumulate the cumulative value.
//!
//!

use std::ops::Mul;

use cosmwasm_std::{Addr, Decimal, Env, QuerierWrapper, Storage, Timestamp, Uint128};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::AbstractResult;

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
    ) -> AbstractResult<()> {
        let block_time = env.block.time;

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
        self.0.save(store, &twa).map_err(Into::into)
    }

    /// Applies the current value to the TWA for the duration since the last update
    /// and returns the cumulative value and block time.
    pub fn accumulate(
        &self,
        env: &Env,
        store: &mut dyn Storage,
        current_value: Decimal,
    ) -> AbstractResult<Option<u128>> {
        let mut twa = self.0.load(store)?;
        let block_time = env.block.time;
        if block_time <= twa.last_block_time {
            return Ok(None);
        }

        let time_elapsed = Uint128::from(block_time.seconds() - twa.last_block_time.seconds());
        twa.last_block_time = block_time;

        if !current_value.is_zero() {
            twa.cumulative_value = twa
                .cumulative_value
                .wrapping_add(time_elapsed.mul(current_value).u128());
        };
        self.0.save(store, &twa)?;
        Ok(Some(twa.cumulative_value))
    }

    pub fn get_value(&self, store: &dyn Storage) -> AbstractResult<Decimal> {
        Ok(self.0.load(store)?.average_value)
    }

    pub fn load(&self, store: &dyn Storage) -> AbstractResult<TimeWeightedAverageData> {
        self.0.load(store).map_err(Into::into)
    }

    pub fn query(
        &self,
        querier: &QuerierWrapper,
        remote_contract_addr: Addr,
    ) -> AbstractResult<TimeWeightedAverageData> {
        self.0
            .query(querier, remote_contract_addr)
            .map_err(Into::into)
    }

    /// Get average value, updates when possible
    pub fn try_update_value(
        &self,
        env: &Env,
        store: &mut dyn Storage,
    ) -> AbstractResult<Option<Decimal>> {
        let mut twa = self.0.load(store)?;

        let block_time = env.block.time;

        let time_elapsed = block_time.seconds() - twa.last_averaging_block_time.seconds();

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
    ) -> AbstractResult<()> {
        let mut twa = self.0.load(store)?;
        twa.averaging_period = averaging_period;
        self.0.save(store, &twa).map_err(Into::into)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TimeWeightedAverageData {
    // settings for accumulating value data
    pub precision: u8,
    pub last_block_time: Timestamp,
    pub cumulative_value: u128,

    // Data to get average value
    pub last_averaging_block_time: Timestamp,
    pub last_averaging_block_height: u64,
    pub last_averaging_cumulative_value: u128,
    pub averaging_period: u64,
    /// The requested average value
    pub average_value: Decimal,
}

impl TimeWeightedAverageData {
    pub fn needs_refresh(&self, env: &Env) -> bool {
        let block_time = env.block.time;

        let time_elapsed = block_time.seconds() - self.last_averaging_block_time.seconds();

        // At least one full period has passed since the last update
        time_elapsed >= self.averaging_period
    }
}
