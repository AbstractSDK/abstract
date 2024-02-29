pub mod error;
pub mod helpers;
pub mod incentives;
pub mod suite;

use abstract_interface::Abstract;
use abstract_interface::ExecuteMsgFns;
use cosmwasm_std::coins;
use cw_orch::{osmosis_test_tube::osmosis_test_tube::Module, prelude::*};
use error::OsmosisPoolError;
use incentives::Incentives;
use osmosis_test_tube::osmosis_std::types::osmosis::poolincentives::v1beta1::QueryLockableDurationsRequest;
use suite::Suite;

pub const EUR_TOKEN: &str = "eur";
pub const EUR_TOKEN_FAST: &str = "eur_fast";
pub const EUR_TOKEN_SLOW: &str = "eur_slow";
pub const USD_TOKEN: &str = "usd";
pub const AXL_USD_TOKEN: &str = "axl_usd";
pub const NUM_EPOCHS_POOL: u64 = 100;

pub const INCENTIVES_AMOUNT: u128 = 100_000_000_000_000;
pub const FAST_INCENTIVES_DENOM: &str = "fast_gauge_incentives";
pub const SLOW_INCENTIVES_DENOM: &str = "slow_gauge_incentives";

pub const OSMOSIS: &str = "osmosis";

pub struct OsmosisPools {
    pub chain: OsmosisTestTube,
    /// Used to create pools, add incentives...
    pub suite: Suite,
    // those are all token denoms
    pub eur_token: String,
    pub eur_token_fast: String,
    pub eur_token_slow: String,
    pub usd_token: String,
    pub axl_usd_token: String,
    // Those are pool ids
    pub eur_usd_pool: u64,
    pub fast_incentivized_eur_usd_pool: u64,
    pub slow_incentivized_eur_usd_pool: u64,
    pub usd_axl_usd_pool: u64,
    // Incentives token denoms
    pub fast_incentives_token: String,
    pub slow_incentives_token: String,
}

impl PartialEq for OsmosisPools {
    fn eq(&self, other: &Self) -> bool {
        self.eur_token == other.eur_token
            && self.eur_token_fast == other.eur_token_fast
            && self.eur_token_slow == other.eur_token_slow
            && self.usd_token == other.usd_token
            && self.eur_usd_pool == other.eur_usd_pool
            && self.fast_incentivized_eur_usd_pool == other.fast_incentivized_eur_usd_pool
            && self.slow_incentivized_eur_usd_pool == other.slow_incentivized_eur_usd_pool
    }
}

impl std::fmt::Debug for OsmosisPools {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OsmosisPools")
            .field("eur_token", &self.eur_token)
            .field("eur_token_fast", &self.eur_token_fast)
            .field("eur_token_slow", &self.eur_token_slow)
            .field("usd_token", &self.usd_token)
            .field("eur_usd_pool", &self.eur_usd_pool)
            .field(
                "fast_incentivized_eur_usd_pool",
                &self.fast_incentivized_eur_usd_pool,
            )
            .field(
                "slow_incentivized_eur_usd_pool",
                &self.slow_incentivized_eur_usd_pool,
            )
            .finish()
    }
}

impl Deploy<OsmosisTestTube> for OsmosisPools {
    type Error = OsmosisPoolError;
    type DeployData = Empty;

    fn store_on(chain: OsmosisTestTube) -> Result<Self, Self::Error> {
        // we create multiple pool types with Native coins, Cw20 and incentives
        let mut suite = Suite::new(chain.clone());

        let abstr = Abstract::load_from(chain.clone())?;

        abstr.ans_host.update_dexes(vec![OSMOSIS.into()], vec![])?;

        // We create a pool
        let eur_usd_pool = suite.create_pool(
            Coin::new(1_000_000, EUR_TOKEN),
            Coin::new(1_100_000, USD_TOKEN),
        )?;
        let usd_axl_usd_pool = suite.create_pool(
            Coin::new(1_000_000, AXL_USD_TOKEN),
            Coin::new(1_000_000, USD_TOKEN),
        )?;
        let fast_incentivized_eur_usd_pool = suite.create_pool(
            Coin::new(1_000_000, EUR_TOKEN_FAST),
            Coin::new(1_100_000, USD_TOKEN),
        )?;
        let slow_incentivized_eur_usd_pool = suite.create_pool(
            Coin::new(1_000_000, EUR_TOKEN_SLOW),
            Coin::new(1_100_000, USD_TOKEN),
        )?;

        // We create incentives for the pools

        let possible_durations = Incentives::new(&*chain.app.borrow())
            .query_lockable_durations(&QueryLockableDurationsRequest {})?;

        suite.incentivize_pool(
            fast_incentivized_eur_usd_pool,
            possible_durations
                .lockable_durations
                .first()
                .unwrap()
                .seconds,
            coins(INCENTIVES_AMOUNT, FAST_INCENTIVES_DENOM),
            NUM_EPOCHS_POOL,
        )?;
        suite.incentivize_pool(
            slow_incentivized_eur_usd_pool,
            possible_durations
                .lockable_durations
                .last()
                .unwrap()
                .seconds,
            coins(INCENTIVES_AMOUNT, SLOW_INCENTIVES_DENOM),
            NUM_EPOCHS_POOL,
        )?;

        Ok(Self {
            chain: chain.clone(),
            eur_token: EUR_TOKEN.to_string(),
            eur_token_fast: EUR_TOKEN_FAST.to_string(),
            eur_token_slow: EUR_TOKEN_SLOW.to_string(),
            usd_token: USD_TOKEN.to_string(),
            axl_usd_token: AXL_USD_TOKEN.to_string(),
            eur_usd_pool,
            fast_incentivized_eur_usd_pool,
            slow_incentivized_eur_usd_pool,
            usd_axl_usd_pool,
            suite,
            fast_incentives_token: FAST_INCENTIVES_DENOM.to_string(),
            slow_incentives_token: SLOW_INCENTIVES_DENOM.to_string(),
        })
    }

    fn deployed_state_file_path() -> Option<String> {
        todo!()
    }

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<OsmosisTestTube>>> {
        todo!()
    }

    fn load_from(_chain: OsmosisTestTube) -> Result<Self, Self::Error> {
        unimplemented!(
            "You can't load Osmosis Pools. You need to pass it around inside your tests instead"
        )
    }
}
