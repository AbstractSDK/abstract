pub mod error;
pub mod helpers;
pub mod incentives;

use std::rc::Rc;

use crate::helpers::osmosis_pool;
use abstract_core::objects::pool_id::PoolAddressBase;
use abstract_core::objects::PoolMetadata;
use abstract_core::objects::{AssetEntry, LpToken};
use abstract_interface::Abstract;
use abstract_interface::ExecuteMsgFns;
use cosmwasm_std::{coin, coins};
use cw_asset::AssetInfo;
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

pub const EUR_TOKEN: &str = "eur";
pub const EUR_TOKEN_FAST: &str = "eur_fast";
pub const EUR_TOKEN_SLOW: &str = "eur_slow";
pub const USD_TOKEN: &str = "usd";
pub const AXL_USD_TOKEN: &str = "axl_usd";
pub const NUM_EPOCHS_POOL: u64 = 100;
pub const POOL_LOCK_FAST: i64 = 3600; // Lock duration is fast
pub const POOL_LOCK_SLOW: i64 = 24 * 3600; // Lock duration is long

pub const INCENTIVES_AMOUNT: u128 = 100_000_000_000_000;
pub const INCENTIVES_DENOM: &str = "gauge_incentives";

pub const OSMOSIS: &str = "osmosis";

pub struct OsmosisPools {
    pub owner: Rc<SigningAccount>,
    pub eur_token: AssetInfo,
    pub eur_token_fast: AssetInfo,
    pub eur_token_slow: AssetInfo,
    pub usd_token: AssetInfo,
    pub axl_usd_token: AssetInfo,
    pub eur_usd_pool: u64,
    pub fast_incentivized_eur_usd_pool: u64,
    pub slow_incentivized_eur_usd_pool: u64,
    pub usd_axl_usd_pool: u64, // pub eur_usd_staking: Addr,
                               // pub raw_eur_staking: Addr,
                               // pub raw_raw_2_staking: Addr,
                               // // incentivized pair
                               // // rewarded in wynd
                               // pub eur_usd_pair: Addr,
                               // pub eur_usd_lp: AbstractCw20Base<MockBech32>,
                               // pub wynd_token: AssetInfo,
                               // pub wynd_eur_pair: Addr,
                               // pub wynd_eur_lp: AbstractCw20Base<MockBech32>,
                               // pub raw_token: AbstractCw20Base<MockBech32>,
                               // pub raw_2_token: AbstractCw20Base<MockBech32>,
                               // pub raw_eur_pair: Addr,
                               // pub raw_eur_lp: AbstractCw20Base<MockBech32>,
                               // pub raw_raw_2_pair: Addr,
                               // pub raw_raw_2_lp: AbstractCw20Base<MockBech32>,
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
    pub fn create_pool(
        chain: OsmosisTestTube,
        liquidity: &[Coin],
        sender: &SigningAccount,
    ) -> Result<u64, OsmosisPoolError> {
        let pool_id = Gamm::new(&*chain.app.borrow())
            .create_basic_pool(liquidity, sender)?
            .data
            .pool_id;
        Ok(pool_id)
    }
    /// registers the WynDex contracts and assets on Abstract
    /// this includes:
    /// - registering the assets on ANS
    ///   - EUR
    ///   - USD
    ///   - WYND
    ///   - RAW
    ///   - RAW_2
    ///   - EUR/USD LP
    ///   - EUR/WYND LP
    ///   - EUR/RAW LP
    ///   - RAW/RAW_2 LP
    /// - Register the staking contract
    ///   - wyndex:staking/wyndex/eur,usd (native)
    ///   - wyndex:staking/wyndex/eur,raw (native-cw20)
    ///   - wyndex:staking/wyndex/raw,raw_2 (cw20-cw20)
    /// - Register the pair contracts
    ///   - wyndex/eur,usd
    ///   - wyndex/eur,wynd
    pub(crate) fn register_info_on_abstract(
        &self,
        abstrct: &Abstract<OsmosisTestTube>,
    ) -> Result<(), CwOrchError> {
        let eur_asset = AssetEntry::new(EUR_TOKEN);
        let eur_fast_asset = AssetEntry::new(EUR_TOKEN_FAST);
        let eur_slow_asset = AssetEntry::new(EUR_TOKEN_SLOW);
        let usd_asset = AssetEntry::new(USD_TOKEN);
        let axl_usd_asset = AssetEntry::new(AXL_USD_TOKEN);

        let eur_usd_lp_asset = LpToken::new(OSMOSIS, vec![EUR_TOKEN, USD_TOKEN]);
        let fast_incentivized_eur_usd_lp_asset =
            LpToken::new(OSMOSIS, vec![EUR_TOKEN_FAST, USD_TOKEN]);
        let slow_incentivized_eur_usd_lp_asset =
            LpToken::new(OSMOSIS, vec![EUR_TOKEN_SLOW, USD_TOKEN]);
        let usd_axl_usd_lp_asset = LpToken::new(OSMOSIS, vec![USD_TOKEN, AXL_USD_TOKEN]);

        // Register addresses on ANS
        abstrct.ans_host.update_asset_addresses(
            vec![
                (
                    eur_asset.to_string(),
                    cw_asset::AssetInfoBase::native(self.eur_token.to_string()),
                ),
                (
                    eur_fast_asset.to_string(),
                    cw_asset::AssetInfoBase::native(self.eur_token_fast.to_string()),
                ),
                (
                    eur_slow_asset.to_string(),
                    cw_asset::AssetInfoBase::native(self.eur_token_slow.to_string()),
                ),
                (
                    usd_asset.to_string(),
                    cw_asset::AssetInfoBase::native(self.usd_token.to_string()),
                ),
                (
                    axl_usd_asset.to_string(),
                    cw_asset::AssetInfoBase::native(self.axl_usd_token.to_string()),
                ),
                (
                    eur_usd_lp_asset.to_string(),
                    cw_asset::AssetInfoBase::native(osmosis_pool(self.eur_usd_pool)),
                ),
                (
                    fast_incentivized_eur_usd_lp_asset.to_string(),
                    cw_asset::AssetInfoBase::native(osmosis_pool(
                        self.fast_incentivized_eur_usd_pool,
                    )),
                ),
                (
                    slow_incentivized_eur_usd_lp_asset.to_string(),
                    cw_asset::AssetInfoBase::native(osmosis_pool(
                        self.slow_incentivized_eur_usd_pool,
                    )),
                ),
                (
                    usd_axl_usd_lp_asset.to_string(),
                    cw_asset::AssetInfoBase::native(osmosis_pool(self.usd_axl_usd_pool)),
                ),
            ],
            vec![],
        )?;

        abstrct
            .ans_host
            .update_dexes(vec![OSMOSIS.into()], vec![])?;

        abstrct.ans_host.update_pools(
            vec![
                (
                    PoolAddressBase::id(self.eur_usd_pool),
                    PoolMetadata::constant_product(
                        OSMOSIS,
                        vec![eur_asset.clone(), usd_asset.clone()],
                    ),
                ),
                (
                    PoolAddressBase::id(self.fast_incentivized_eur_usd_pool),
                    PoolMetadata::constant_product(
                        OSMOSIS,
                        vec![eur_fast_asset.clone(), usd_asset.clone()],
                    ),
                ),
                (
                    PoolAddressBase::id(self.slow_incentivized_eur_usd_pool),
                    PoolMetadata::constant_product(
                        OSMOSIS,
                        vec![eur_slow_asset.clone(), usd_asset.clone()],
                    ),
                ),
                (
                    PoolAddressBase::id(self.usd_axl_usd_pool),
                    PoolMetadata::constant_product(OSMOSIS, vec![axl_usd_asset.clone(), usd_asset]),
                ),
            ],
            vec![],
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

        // We create a pool
        let eur_usd_pool = OsmosisPools::create_pool(
            chain.clone(),
            &[
                Coin::new(1_000_000, EUR_TOKEN),
                Coin::new(1_100_000, USD_TOKEN),
            ],
            &owner,
        )?;
        let usd_axl_usd_pool = OsmosisPools::create_pool(
            chain.clone(),
            &[
                Coin::new(1_000_000, AXL_USD_TOKEN),
                Coin::new(1_000_000, USD_TOKEN),
            ],
            &owner,
        )?;
        let fast_incentivized_eur_usd_pool = OsmosisPools::create_pool(
            chain.clone(),
            &[
                Coin::new(1_000_000, EUR_TOKEN_FAST),
                Coin::new(1_100_000, USD_TOKEN),
            ],
            &owner,
        )?;
        let slow_incentivized_eur_usd_pool = OsmosisPools::create_pool(
            chain.clone(),
            &[
                Coin::new(1_000_000, EUR_TOKEN_SLOW),
                Coin::new(1_100_000, USD_TOKEN),
            ],
            &owner,
        )?;

        // We create incentives for the pool
        let current_block = chain.block_info()?;
        Incentives::new(&*chain.app.borrow()).create_gauge(
            MsgCreateGauge {
                is_perpetual: false,
                owner: owner.address(),
                distribute_to: Some(QueryCondition {
                    lock_query_type: LockQueryType::ByDuration as i32,
                    duration: Some(Duration {
                        seconds: POOL_LOCK_FAST,
                        nanos: 0,
                    }),
                    denom: format!("gamm/pool/{}", fast_incentivized_eur_usd_pool),
                    timestamp: None,
                }),
                coins: cosmwasm_to_proto_coins(coins(INCENTIVES_AMOUNT, INCENTIVES_DENOM)),
                start_time: Some(Timestamp {
                    seconds: current_block.time.seconds() as i64,
                    nanos: current_block.time.subsec_nanos() as i32,
                }),
                num_epochs_paid_over: NUM_EPOCHS_POOL,
                pool_id: 0,
            },
            &owner,
        )?;

        let osmosis = Self {
            owner,
            eur_token: AssetInfo::native(EUR_TOKEN),
            eur_token_fast: AssetInfo::native(EUR_TOKEN_FAST),
            eur_token_slow: AssetInfo::native(EUR_TOKEN_SLOW),
            usd_token: AssetInfo::native(USD_TOKEN),
            eur_usd_pool,
            fast_incentivized_eur_usd_pool,
            slow_incentivized_eur_usd_pool,
            axl_usd_token: AssetInfo::native(AXL_USD_TOKEN),
            usd_axl_usd_pool,
        };
        // register contracts in abstract host
        let abstract_ = Abstract::load_from(chain)?;
        osmosis.register_info_on_abstract(&abstract_)?;

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
