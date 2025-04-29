#![cfg(feature = "osmosis-test")]

mod osmosis_test {

    use abstract_adapter::abstract_interface::{
        Abstract, AbstractInterfaceError, AccountI, AdapterDeployer, DeployStrategy,
    };
    use abstract_adapter::std::{
        ans_host::ExecuteMsgFns,
        objects::{pool_id::PoolAddressBase, AnsAsset, AssetEntry, PoolMetadata, PoolType},
    };
    use abstract_client::GovernanceDetails;
    use abstract_cw_staking::interface::CwStakingAdapter;
    use abstract_cw_staking::{contract::CONTRACT_VERSION, msg::StakingQueryMsgFns};
    use abstract_staking_standard::{
        msg::{StakingInfo, StakingInfoResponse},
        CwStakingError,
    };
    use cosmwasm_std::{coin, coins, from_json, to_json_binary, Empty, Uint128};
    use cw_asset::AssetInfoBase;
    use cw_orch_osmosis_test_tube::osmosis_test_tube::{
        osmosis_std::{
            shim::Timestamp,
            types::osmosis::{
                incentives::{MsgAddToGauge, MsgCreateGauge, QueryLockableDurationsRequest},
                lockup::{
                    AccountLockedCoinsRequest, AccountLockedCoinsResponse, LockQueryType,
                    QueryCondition,
                },
                poolincentives::v1beta1::QueryGaugeIdsRequest,
            },
        },
        Module, Runner,
    };
    use cw_orch_osmosis_test_tube::OsmosisTestTube;

    use cw_orch::prelude::*;

    const OSMOSIS: &str = "osmosis";
    const DENOM: &str = "uosmo";

    const ASSET_1: &str = DENOM;
    const ASSET_2: &str = "uatom";

    pub const LP: &str = "osmosis/osmo,atom";

    use abstract_cw_staking::CW_STAKING_ADAPTER_ID;

    fn get_pool_token(id: u64) -> String {
        format!("gamm/pool/{}", id)
    }

    fn setup_osmosis() -> anyhow::Result<(
        OsmosisTestTube,
        u64,
        CwStakingAdapter<OsmosisTestTube>,
        AccountI<OsmosisTestTube>,
    )> {
        std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::try_init();
        let tube = OsmosisTestTube::new(vec![
            coin(1_000_000_000_000, ASSET_2),
            coin(1_000_000_000_000, ASSET_1),
        ]);

        let deployment = Abstract::deploy_on(tube.clone(), ())?;

        let _root_os = AccountI::create_default_account(
            &deployment,
            GovernanceDetails::Monarchy {
                monarch: tube.sender_addr().to_string(),
            },
        )?;
        let staking: CwStakingAdapter<OsmosisTestTube> =
            CwStakingAdapter::new(CW_STAKING_ADAPTER_ID, tube.clone());

        staking.deploy(CONTRACT_VERSION.parse()?, Empty {}, DeployStrategy::Error)?;

        let os = AccountI::create_default_account(
            &deployment,
            GovernanceDetails::Monarchy {
                monarch: tube.sender_addr().to_string(),
            },
        )?;

        // transfer some LP tokens to the AccountI, as if it provided liquidity
        let pool_id = tube.create_pool(vec![coin(1_000, ASSET_1), coin(1_000, ASSET_2)])?;

        deployment
            .ans_host
            .update_asset_addresses(
                vec![
                    ("osmo".to_string(), cw_asset::AssetInfoBase::native(ASSET_1)),
                    ("atom".to_string(), cw_asset::AssetInfoBase::native(ASSET_2)),
                    (
                        LP.to_string(),
                        cw_asset::AssetInfoBase::native(get_pool_token(pool_id)),
                    ),
                ],
                vec![],
            )
            .unwrap();

        deployment
            .ans_host
            .update_dexes(vec![OSMOSIS.into()], vec![])
            .unwrap();

        deployment
            .ans_host
            .update_pools(
                vec![(
                    PoolAddressBase::id(pool_id),
                    PoolMetadata::constant_product(
                        OSMOSIS,
                        vec!["osmo".to_string(), "atom".to_string()],
                    ),
                )],
                vec![],
            )
            .unwrap();

        // install exchange on AccountI
        os.install_adapter(&staking, &[])?;

        tube.bank_send(os.addr_str()?, coins(1_000u128, get_pool_token(pool_id)))?;

        Ok((tube, pool_id, staking, os))
    }

    #[test]
    fn staking_inited() -> anyhow::Result<()> {
        let (_, pool_id, staking, _) = setup_osmosis()?;

        // query staking info
        let staking_info = staking.info(OSMOSIS.into(), vec![AssetEntry::new(LP)])?;
        let staking_coin = AssetInfoBase::native(get_pool_token(pool_id));
        assert_eq!(
            staking_info,
            StakingInfoResponse {
                infos: vec![StakingInfo {
                    staking_target: pool_id.into(),
                    staking_token: staking_coin.clone(),
                    unbonding_periods: Some(vec![]),
                    max_claims: None,
                }],
            }
        );

        Ok(())
    }

    #[test]
    fn stake_lp() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let account_addr = os.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 stake-coins
        staking.stake(AnsAsset::new(LP, 100u128), OSMOSIS.into(), dur, &os)?;

        tube.wait_seconds(10000)?;
        // query stake
        staking.staked(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
            dur,
        )?;

        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert_eq!(staked_balance.coins[0].amount, 100u128.to_string());

        Ok(())
    }

    #[test]
    fn unstake_lp() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let account_addr = os.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        staking.stake(AnsAsset::new(LP, 100u128), OSMOSIS.into(), dur, &os)?;

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert_eq!(staked_balance.coins[0].amount, 100u128.to_string());

        // now unbond 50
        staking.unstake(AnsAsset::new(LP, 50u128), OSMOSIS.into(), dur, &os)?;
        // query unbond
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_eq!(unbonding.claims[0][0].amount, Uint128::new(50));

        // Wait, and check unbonding status
        tube.wait_seconds(2)?;
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert!(unbonding.claims[0].is_empty());

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert_eq!(staked_balance.coins[0].amount, 50u128.to_string());
        Ok(())
    }

    #[test]
    fn claim_all() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let account_addr = os.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        staking.stake(AnsAsset::new(LP, 100u128), OSMOSIS.into(), dur, &os)?;

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert_eq!(staked_balance.coins[0].amount, 100u128.to_string());

        // now unbond all
        staking.claim(AssetEntry::new(LP), OSMOSIS.into(), &os)?;
        // query unbond
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_eq!(unbonding.claims[0][0].amount, Uint128::new(100));

        // Wait, and check unbonding status
        tube.wait_seconds(2)?;
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert!(unbonding.claims[0].is_empty());

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert!(staked_balance.coins.is_empty());
        Ok(())
    }

    // Currently not supported for provide/withdraw
    #[test]
    fn concentrated_liquidity() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;

        let lp = "osmosis/osmo2,atom2";
        // transfer some LP tokens to the AccountI, as if it provided liquidity
        let pool_id = tube.create_pool(vec![coin(1_000, ASSET_1), coin(1_000, ASSET_2)])?;

        let deployment = Abstract::load_from(tube.clone())?;
        deployment
            .ans_host
            .update_asset_addresses(
                vec![
                    (
                        "osmo2".to_string(),
                        cw_asset::AssetInfoBase::native(ASSET_1),
                    ),
                    (
                        "atom2".to_string(),
                        cw_asset::AssetInfoBase::native(ASSET_2),
                    ),
                    (
                        lp.to_string(),
                        cw_asset::AssetInfoBase::native(get_pool_token(pool_id)),
                    ),
                ],
                vec![],
            )
            .unwrap();
        deployment
            .ans_host
            .update_pools(
                vec![(
                    PoolAddressBase::id(pool_id),
                    PoolMetadata::concentrated_liquidity(
                        OSMOSIS,
                        vec!["osmo2".to_string(), "atom2".to_string()],
                    ),
                )],
                vec![],
            )
            .unwrap();
        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        let err = staking
            .stake(AnsAsset::new(lp, 100u128), OSMOSIS.into(), dur, &os)
            .unwrap_err();
        if let AbstractInterfaceError::Orch(CwOrchError::StdErr(e)) = err {
            let expected_err = CwStakingError::NotSupportedPoolType(
                PoolType::ConcentratedLiquidity.to_string(),
                "osmosis".to_owned(),
            );
            assert!(e.contains(&expected_err.to_string()));
        } else {
            panic!("Expected stderror");
        };
        Ok(())
    }

    #[test]
    fn reward_tokens_add_to_gauge() -> anyhow::Result<()> {
        let (mut chain, pool_id, staking, _account) = setup_osmosis()?;
        // For gauge
        chain.add_balance(&chain.sender_addr(), coins(1_000_000_000_000, ASSET_1))?;

        let test_tube = chain.app.borrow();
        let incentives = super::incentives::Incentives::new(&*test_tube);
        let poolincentives = super::poolincentives::Poolincentives::new(&*test_tube);

        // Check that we have empty assets at start
        let res = staking.reward_tokens(OSMOSIS.to_owned(), vec![AssetEntry::new(LP)])?;
        assert_eq!(res.tokens, [[]]);

        // Select gauge to refill
        let gauge_ids = poolincentives.query_gauge_ids(&QueryGaugeIdsRequest { pool_id })?;
        let gauge_id_for_refill = gauge_ids.gauge_ids_with_duration[0].gauge_id;

        // Now incentivize pool
        let _time = test_tube.get_block_timestamp().plus_seconds(5);
        let _lockable_durations =
            incentives.query_lockable_durations(&QueryLockableDurationsRequest {})?;
        incentives.add_to_gauge(
            MsgAddToGauge {
                owner: chain.sender_addr().to_string(),
                gauge_id: gauge_id_for_refill,
                rewards: vec![cw_orch_osmosis_test_tube::osmosis_test_tube::osmosis_std::types::cosmos::base::v1beta1::Coin {
                    denom: ASSET_1.to_owned(),
                    amount: "100000000".to_owned(),
                }],
            },
            &chain.sender,
        )?;
        chain.wait_seconds(10)?;

        let res = staking.reward_tokens(OSMOSIS.to_owned(), vec![AssetEntry::new(LP)])?;
        assert_eq!(res.tokens, [[AssetInfoBase::Native(ASSET_1.to_owned())]]);
        Ok(())
    }

    #[test]
    #[ignore = "Currently broken, see https://github.com/osmosis-labs/test-tube/pull/53"]
    fn reward_tokens_create_gauge() -> anyhow::Result<()> {
        let (mut chain, pool_id, staking, _account) = setup_osmosis()?;
        // For gauge
        chain.add_balance(&chain.sender_addr(), coins(1_000_000_000_000, ASSET_1))?;

        let test_tube = chain.app.borrow();
        let incentives = super::incentives::Incentives::new(&*test_tube);
        let _poolincentives = super::poolincentives::Poolincentives::new(&*test_tube);

        // Check that we have empty assets at start
        let res = staking.reward_tokens(OSMOSIS.to_owned(), vec![AssetEntry::new(LP)])?;
        assert_eq!(res.tokens, [[]]);

        // Now incentivize pool
        let time = test_tube.get_block_timestamp().plus_seconds(5);
        let lockable_durations =
            incentives.query_lockable_durations(&QueryLockableDurationsRequest {})?;
        incentives.create_gauge(
            MsgCreateGauge {
                pool_id,
                is_perpetual: true,
                owner: chain.sender_addr().to_string(),
                distribute_to: Some(QueryCondition {
                    lock_query_type: LockQueryType::ByDuration.into(),
                    denom: get_pool_token(pool_id),
                    duration: Some(lockable_durations.lockable_durations[0].clone()),
                    timestamp: None,
                }),
                coins: vec![cw_orch_osmosis_test_tube::osmosis_test_tube::osmosis_std::types::cosmos::base::v1beta1::Coin {
                    denom: ASSET_1.to_owned(),
                    amount: "100000000".to_owned(),
                }],
                start_time: Some(Timestamp {
                    seconds: time.seconds() as i64,
                    nanos: 0,
                }),
                num_epochs_paid_over: 1,
            },
            &chain.sender,
        )?;
        chain.wait_seconds(10)?;

        let res = staking.reward_tokens(OSMOSIS.to_owned(), vec![AssetEntry::new(LP)])?;
        assert_eq!(
            res.tokens,
            [vec![], vec![AssetInfoBase::Native(ASSET_1.to_owned())]]
        );
        Ok(())
    }

    #[test]
    #[ignore = "Tracking how deserialization is managed as it's broken in v24"]
    fn deserialize_gauge_by_id_response() {
        use serde_cw_value::Value::*;
        let value = Map([(
            String("gauge".to_owned()),
            Map([
                (String("id".to_owned()), String("1".to_owned())),
                (String("is_perpetual".to_owned()), Bool(true)),
                (
                    String("distribute_to".to_string()),
                    Map([
                        (
                            String("denom".to_string()),
                            String("gamm/pool/1".to_string()),
                        ),
                        (String("duration".to_string()), String("3600s".to_string())),
                        (
                            String("lock_query_type".to_string()),
                            String("ByDuration".to_string()),
                        ),
                        (
                            String("timestamp".to_string()),
                            String("0001-01-01T00:00:00Z".to_string()),
                        ),
                    ]
                    .into_iter()
                    .collect()),
                ),
                (String("coins".to_owned()), Seq(vec![])),
                (
                    String("start_time".to_owned()),
                    String("2024-05-01T15:09:38.190576621Z".to_owned()),
                ),
                (
                    String("num_epochs_paid_over".to_owned()),
                    String("1".to_owned()),
                ),
                (String("filled_epochs".to_owned()), String("0".to_owned())),
                (String("distributed_coins".to_owned()), Seq(vec![])),
            ]
            .into_iter()
            .collect()),
        )]
        .into_iter()
        .collect());

        let bin = to_json_binary(&value).unwrap();
        let gauge_by_id_response = from_json::<cw_orch_osmosis_test_tube::osmosis_test_tube::osmosis_std::types::osmosis::incentives::GaugeByIdResponse>(bin);
        assert!(gauge_by_id_response.is_err());
    }
}

#[allow(unused)]
pub mod incentives {
    use cw_orch_osmosis_test_tube::osmosis_test_tube::osmosis_std::types::osmosis::incentives::{
        GaugeByIdRequest, GaugeByIdResponse, MsgAddToGauge, MsgAddToGaugeResponse, MsgCreateGauge,
        MsgCreateGaugeResponse, QueryLockableDurationsRequest, QueryLockableDurationsResponse,
    };
    use cw_orch_osmosis_test_tube::osmosis_test_tube::{fn_execute, fn_query, Module, Runner};

    pub struct Incentives<'a, R: Runner<'a>> {
        runner: &'a R,
    }

    impl<'a, R: Runner<'a>> Module<'a, R> for Incentives<'a, R> {
        fn new(runner: &'a R) -> Self {
            Self { runner }
        }
    }

    impl<'a, R> Incentives<'a, R>
    where
        R: Runner<'a>,
    {
        fn_execute! {
            pub create_gauge: MsgCreateGauge => MsgCreateGaugeResponse
        }

        fn_execute! {
            pub add_to_gauge: MsgAddToGauge => MsgAddToGaugeResponse
        }

        fn_query! {
            pub query_lockable_durations ["/osmosis.incentives.Query/LockableDurations"]: QueryLockableDurationsRequest => QueryLockableDurationsResponse
        }

        fn_query! {
            pub query_gauge_by_id ["/osmosis.incentives.Query/GaugeByID"]: GaugeByIdRequest => GaugeByIdResponse
        }
    }
}

#[allow(unused)]
pub mod poolincentives {
    use cw_orch_osmosis_test_tube::osmosis_test_tube::{
        fn_execute, fn_query,
        osmosis_std::types::osmosis::poolincentives::v1beta1::{
            QueryGaugeIdsRequest, QueryGaugeIdsResponse,
        },
        Module, Runner,
    };

    pub struct Poolincentives<'a, R: Runner<'a>> {
        runner: &'a R,
    }

    impl<'a, R: Runner<'a>> Module<'a, R> for Poolincentives<'a, R> {
        fn new(runner: &'a R) -> Self {
            Self { runner }
        }
    }

    impl<'a, R> Poolincentives<'a, R>
    where
        R: Runner<'a>,
    {
        fn_query! {
            pub query_gauge_ids ["/osmosis.poolincentives.v1beta1.Query/GaugeIds"]: QueryGaugeIdsRequest => QueryGaugeIdsResponse
        }
    }
}
