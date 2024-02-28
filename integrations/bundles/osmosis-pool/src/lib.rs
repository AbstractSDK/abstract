pub mod error;
pub mod helpers;
pub mod incentives;

use std::rc::Rc;

use crate::helpers::osmosis_pool_token;
use abstract_core::objects::pool_id::PoolAddressBase;
use abstract_core::objects::LpToken;
use abstract_core::objects::PoolMetadata;
use abstract_interface::Abstract;
use abstract_interface::ExecuteMsgFns;
use cosmwasm_std::{coin, coins};
use cw_orch::{
    osmosis_test_tube::osmosis_test_tube::{
        osmosis_std::{
            cosmwasm_to_proto_coins,
            shim::Timestamp,
            types::osmosis::{
                incentives::MsgCreateGauge,
                lockup::{LockQueryType, QueryCondition},
            },
        },
        Account, Gamm, Module, SigningAccount,
    },
    prelude::*,
};
use error::OsmosisPoolError;
use incentives::Incentives;
use osmosis_test_tube::osmosis_std::shim::Duration;
use osmosis_test_tube::osmosis_std::types::osmosis::poolincentives::v1beta1::QueryLockableDurationsRequest;

pub const EUR_TOKEN: &str = "eur";
pub const EUR_TOKEN_FAST: &str = "eur_fast";
pub const EUR_TOKEN_SLOW: &str = "eur_slow";
pub const USD_TOKEN: &str = "usd";
pub const AXL_USD_TOKEN: &str = "axl_usd";
pub const NUM_EPOCHS_POOL: u64 = 100;
pub const POOL_LOCK_FAST: i64 = 60; // Lock duration is fast
pub const POOL_LOCK_SLOW: i64 = 3600; // Lock duration is long

pub const INCENTIVES_AMOUNT: u128 = 100_000_000_000_000;
pub const INCENTIVES_DENOM: &str = "gauge_incentives";

pub const OSMOSIS: &str = "osmosis";

pub struct OsmosisPools {
    pub chain: OsmosisTestTube,
    pub owner: Rc<SigningAccount>,
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
}

impl PartialEq for OsmosisPools {
    fn eq(&self, other: &Self) -> bool {
        self.owner.address() == other.owner.address()
            && self.eur_token == other.eur_token
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
            .field("owner", &self.owner.address())
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

impl OsmosisPools {
    pub fn create_pool(&mut self, asset1: Coin, asset2: Coin) -> Result<u64, OsmosisPoolError> {
        self.chain.add_balance(
            self.owner.address(),
            [asset1.clone(), asset2.clone()].to_vec(),
        )?;
        let pool_id = Gamm::new(&*self.chain.app.borrow())
            .create_basic_pool(&[asset1.clone(), asset2.clone()], &self.owner)?
            .data
            .pool_id;

        // We register assets and the pool inside ANS
        let abstr = Abstract::load_from(self.chain.clone())?;
        abstr.ans_host.update_asset_addresses(
            vec![
                (
                    asset1.denom.clone(),
                    cw_asset::AssetInfoBase::native(asset1.denom.clone()),
                ),
                (
                    asset2.denom.clone(),
                    cw_asset::AssetInfoBase::native(asset2.denom.clone()),
                ),
                (
                    LpToken::new(OSMOSIS, vec![asset1.denom.clone(), asset2.denom.clone()])
                        .to_string(),
                    cw_asset::AssetInfoBase::native(osmosis_pool_token(pool_id)),
                ),
            ],
            vec![],
        )?;
        abstr.ans_host.update_pools(
            vec![(
                PoolAddressBase::id(pool_id),
                PoolMetadata::constant_product(
                    OSMOSIS,
                    vec![asset1.denom.clone(), asset2.denom.clone()],
                ),
            )],
            vec![],
        )?;

        Ok(pool_id)
    }
    pub fn incentivize_pool(
        &mut self,
        pool_id: u64,
        lock_seconds: i64,
        incentives: Vec<Coin>,
        num_epochs: u64,
    ) -> Result<(), OsmosisPoolError> {
        let current_block = self.chain.block_info()?;

        self.chain
            .add_balance(self.owner.address(), incentives.clone())?;

        Incentives::new(&*self.chain.app.borrow()).create_gauge(
            MsgCreateGauge {
                is_perpetual: false,
                owner: self.owner.address(),
                distribute_to: Some(QueryCondition {
                    lock_query_type: LockQueryType::ByDuration as i32,
                    duration: Some(Duration {
                        seconds: lock_seconds,
                        nanos: 0,
                    }),
                    denom: osmosis_pool_token(pool_id),
                    timestamp: None,
                }),
                coins: cosmwasm_to_proto_coins(incentives),
                start_time: Some(Timestamp {
                    seconds: current_block.time.seconds() as i64,
                    nanos: current_block.time.subsec_nanos() as i32,
                }),
                num_epochs_paid_over: num_epochs,
                pool_id: 0,
            },
            &self.owner,
        )?;

        Ok(())
    }
}

impl Deploy<OsmosisTestTube> for OsmosisPools {
    type Error = OsmosisPoolError;
    type DeployData = Empty;

    fn store_on(mut chain: OsmosisTestTube) -> Result<Self, Self::Error> {
        // we create multiple pool types with Native coins, Cw20 and incentives

        // We create a test account with a lot of funds
        let owner = chain.init_account(vec![
            coin(100_000_000_000_000, "uosmo"),
            coin(100_000_000_000_000, EUR_TOKEN),
            coin(100_000_000_000_000, EUR_TOKEN_FAST),
            coin(100_000_000_000_000, EUR_TOKEN_SLOW),
            coin(100_000_000_000_000, USD_TOKEN),
            coin(100_000_000_000_000, AXL_USD_TOKEN),
            coin(100_000_000_000_000, INCENTIVES_DENOM),
        ])?;

        let mut osmosis = Self {
            chain: chain.clone(),
            owner,
            eur_token: EUR_TOKEN.to_string(),
            eur_token_fast: EUR_TOKEN_FAST.to_string(),
            eur_token_slow: EUR_TOKEN_SLOW.to_string(),
            usd_token: USD_TOKEN.to_string(),
            axl_usd_token: AXL_USD_TOKEN.to_string(),
            eur_usd_pool: 0,
            fast_incentivized_eur_usd_pool: 0,
            slow_incentivized_eur_usd_pool: 0,
            usd_axl_usd_pool: 0,
        };

        let abstr = Abstract::load_from(chain.clone())?;

        abstr.ans_host.update_dexes(vec![OSMOSIS.into()], vec![])?;

        // We create a pool
        let eur_usd_pool = osmosis.create_pool(
            Coin::new(1_000_000, EUR_TOKEN),
            Coin::new(1_100_000, USD_TOKEN),
        )?;
        let usd_axl_usd_pool = osmosis.create_pool(
            Coin::new(1_000_000, AXL_USD_TOKEN),
            Coin::new(1_000_000, USD_TOKEN),
        )?;
        let fast_incentivized_eur_usd_pool = osmosis.create_pool(
            Coin::new(1_000_000, EUR_TOKEN_FAST),
            Coin::new(1_100_000, USD_TOKEN),
        )?;
        let slow_incentivized_eur_usd_pool = osmosis.create_pool(
            Coin::new(1_000_000, EUR_TOKEN_SLOW),
            Coin::new(1_100_000, USD_TOKEN),
        )?;
        osmosis.eur_usd_pool = eur_usd_pool;
        osmosis.usd_axl_usd_pool = usd_axl_usd_pool;
        osmosis.fast_incentivized_eur_usd_pool = fast_incentivized_eur_usd_pool;
        osmosis.slow_incentivized_eur_usd_pool = slow_incentivized_eur_usd_pool;

        // We create incentives for the pools

        let possible_durations = Incentives::new(&*chain.app.borrow())
            .query_lockable_durations(&QueryLockableDurationsRequest {})?;

        osmosis.incentivize_pool(
            fast_incentivized_eur_usd_pool,
            possible_durations
                .lockable_durations
                .first()
                .unwrap()
                .seconds,
            coins(INCENTIVES_AMOUNT, INCENTIVES_DENOM),
            NUM_EPOCHS_POOL,
        )?;
        osmosis.incentivize_pool(
            slow_incentivized_eur_usd_pool,
            possible_durations
                .lockable_durations
                .last()
                .unwrap()
                .seconds,
            coins(INCENTIVES_AMOUNT, INCENTIVES_DENOM),
            NUM_EPOCHS_POOL,
        )?;

        Ok(osmosis)
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
