use cosmwasm_std::Decimal;
use cw_orch::osmosis_test_tube::OsmosisTestTube;
use osmosis_test_tube::cosmrs::proto::prost::Message;
use osmosis_test_tube::osmosis_std::types::osmosis::concentratedliquidity::v1beta1::CreateConcentratedLiquidityPoolsProposal;
use osmosis_test_tube::osmosis_std::types::osmosis::concentratedliquidity::v1beta1::MsgCreatePosition;
use osmosis_test_tube::osmosis_std::types::osmosis::concentratedliquidity::v1beta1::Pool;
use osmosis_test_tube::osmosis_std::types::osmosis::concentratedliquidity::v1beta1::PoolRecord;
use osmosis_test_tube::osmosis_std::types::osmosis::concentratedliquidity::v1beta1::PoolsRequest;
use osmosis_test_tube::ConcentratedLiquidity;
use osmosis_test_tube::GovWithAppAccess;

use crate::error::OsmosisPoolError;
use crate::helpers::osmosis_pool_token;
use crate::incentives::Incentives;
use crate::OSMOSIS;
use abstract_core::objects::pool_id::PoolAddressBase;
use abstract_core::objects::LpToken;
use abstract_core::objects::PoolMetadata;
use abstract_interface::Abstract;
use abstract_interface::ExecuteMsgFns;
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
        Gamm, Module,
    },
    prelude::*,
};
use osmosis_test_tube::osmosis_std::shim::Duration;

pub const TICK_SPACING: u64 = 100;
pub const SPREAD_FACTOR: u64 = 1;

pub const INITIAL_LOWER_TICK: i64 = -10000;
pub const INITIAL_UPPER_TICK: i64 = 1000;

pub struct Suite {
    chain: OsmosisTestTube,
}

impl Suite {
    pub fn new(chain: OsmosisTestTube) -> Self {
        Self { chain }
    }

    pub fn create_pool(&mut self, asset1: Coin, asset2: Coin) -> Result<u64, OsmosisPoolError> {
        self.chain.add_balance(
            self.chain.sender(),
            [asset1.clone(), asset2.clone()].to_vec(),
        )?;
        let pool_id = Gamm::new(&*self.chain.app.borrow())
            .create_basic_pool(&[asset1.clone(), asset2.clone()], &self.chain.sender)?
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

    pub fn create_concentrated_liquidity_pool(
        &mut self,
        asset1: Coin,
        asset2: Coin,
        asset1_ans: Option<&str>,
        asset2_ans: Option<&str>,
    ) -> Result<u64, OsmosisPoolError> {
        self.chain.add_balance(
            self.chain.sender(),
            [asset1.clone(), asset2.clone()].to_vec(),
        )?;
        // We need to create a proposal to create the pool
        GovWithAppAccess::new(&self.chain.app.borrow()).propose_and_execute(
            CreateConcentratedLiquidityPoolsProposal::TYPE_URL.to_string(),
            CreateConcentratedLiquidityPoolsProposal {
                title: "Create concentrated uosmo:usdc pool".to_string(),
                description: "Create concentrated uosmo:usdc pool, so that we can trade it"
                    .to_string(),
                pool_records: vec![PoolRecord {
                    denom0: asset1.denom.clone(),
                    denom1: asset2.denom.clone(),
                    tick_spacing: TICK_SPACING,
                    spread_factor: Decimal::percent(SPREAD_FACTOR).atomics().to_string(),
                }],
            },
            self.chain.sender().to_string(),
            &self.chain.sender,
        )?;
        let test_tube = self.chain.app.borrow();
        let cl = ConcentratedLiquidity::new(&*test_tube);

        let pools = cl.query_pools(&PoolsRequest { pagination: None }).unwrap();

        let pool = Pool::decode(pools.pools[0].value.as_slice()).unwrap();
        cl.create_position(
            MsgCreatePosition {
                pool_id: pool.id,
                sender: self.chain.sender().to_string(),
                lower_tick: INITIAL_LOWER_TICK,
                upper_tick: INITIAL_UPPER_TICK,
                tokens_provided: cosmwasm_to_proto_coins(vec![asset1.clone(), asset2.clone()]),
                token_min_amount0: "0".to_string(),
                token_min_amount1: "0".to_string(),
            },
            &self.chain.sender,
        )?;

        // We register assets and the pool inside ANS
        let abstr = Abstract::load_from(self.chain.clone())?;
        let asset1_ans = asset1_ans.unwrap_or(&asset1.denom).to_owned();
        let asset2_ans = asset2_ans.unwrap_or(&asset2.denom).to_owned();

        abstr.ans_host.update_asset_addresses(
            vec![
                (
                    asset1_ans.clone(),
                    cw_asset::AssetInfoBase::native(asset1.denom.clone()),
                ),
                (
                    asset2_ans.clone(),
                    cw_asset::AssetInfoBase::native(asset2.denom.clone()),
                ),
                (
                    LpToken::new(OSMOSIS, vec![asset1_ans.clone(), asset2_ans.clone()]).to_string(),
                    cw_asset::AssetInfoBase::native(osmosis_pool_token(pool.id)),
                ),
            ],
            vec![],
        )?;
        abstr.ans_host.update_pools(
            vec![(
                PoolAddressBase::id(pool.id),
                PoolMetadata::concentrated_liquidity(
                    OSMOSIS,
                    vec![asset1_ans.clone(), asset2_ans.clone()],
                ),
            )],
            vec![],
        )?;

        Ok(pool.id)
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
            .add_balance(self.chain.sender(), incentives.clone())?;

        Incentives::new(&*self.chain.app.borrow()).create_gauge(
            MsgCreateGauge {
                is_perpetual: false,
                owner: self.chain.sender().to_string(),
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
            &self.chain.sender,
        )?;

        Ok(())
    }
}
