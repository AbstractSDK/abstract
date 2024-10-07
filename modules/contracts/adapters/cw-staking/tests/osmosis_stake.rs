#![allow(unused)]
#![cfg(feature = "osmosis-test")]

mod common;

mod osmosis_test {
    use std::path::PathBuf;

    use abstract_adapter::abstract_interface::{
        Abstract, AbstractAccount, AbstractInterfaceError, AdapterDeployer, DeployStrategy,
        Account, RegisteredModule,
    };
    use abstract_adapter::objects::dependency::StaticDependency;
    use abstract_adapter::std::{
        adapter,
        ans_host::ExecuteMsgFns,
        objects::{
            pool_id::PoolAddressBase, AccountId, AnsAsset, AssetEntry, PoolMetadata, PoolType,
        },
        MANAGER,
    };
    use abstract_adapter::traits::{Dependencies, ModuleIdentification};
    use abstract_cw_staking::{
        contract::CONTRACT_VERSION,
        msg::{
            ExecuteMsg, InstantiateMsg, QueryMsg, RewardTokensResponse, StakingAction,
            StakingExecuteMsg, StakingQueryMsgFns,
        },
    };
    use abstract_staking_standard::{
        msg::{StakingInfo, StakingInfoResponse},
        CwStakingError,
    };
    use cosmwasm_std::{coin, coins, from_json, to_json_binary, Addr, Empty, Uint128};
    use cw_asset::AssetInfoBase;
    use cw_orch_osmosis_test_tube::osmosis_test_tube::{
        osmosis_std::{
            shim::{Duration, Timestamp},
            types::osmosis::{
                incentives::{MsgAddToGauge, MsgCreateGauge, QueryLockableDurationsRequest},
                lockup::{
                    AccountLockedCoinsRequest, AccountLockedCoinsResponse, LockQueryType,
                    QueryCondition,
                },
                poolincentives::v1beta1::QueryGaugeIdsRequest,
            },
        },
        Module, OsmosisTestApp, Runner,
    };
    use cw_orch_osmosis_test_tube::OsmosisTestTube;

    use cw_orch::{interface, prelude::*};
    use speculoos::prelude::*;

    const OSMOSIS: &str = "osmosis";
    const DENOM: &str = "uosmo";

    const ASSET_1: &str = DENOM;
    const ASSET_2: &str = "uatom";

    pub const LP: &str = "osmosis/osmo,atom";

    use abstract_cw_staking::CW_STAKING_ADAPTER_ID;

    use crate::common::create_default_account;

    fn get_pool_token(id: u64) -> String {
        format!("gamm/pool/{}", id)
    }

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct OsmosisStakingAdapter<Chain>;

    impl<Chain: CwEnv> AdapterDeployer<Chain, Empty> for OsmosisStakingAdapter<Chain> {}

    impl<Chain: CwEnv> RegisteredModule for OsmosisStakingAdapter<Chain> {
        type InitMsg = InstantiateMsg;

        fn module_id<'a>() -> &'a str {
            abstract_cw_staking::contract::CW_STAKING_ADAPTER.module_id()
        }

        fn module_version<'a>() -> &'a str {
            abstract_cw_staking::contract::CW_STAKING_ADAPTER.version()
        }

        fn dependencies<'a>() -> &'a [StaticDependency] {
            abstract_cw_staking::contract::CW_STAKING_ADAPTER.dependencies()
        }
    }

    impl<Chain: CwEnv> Uploadable for OsmosisStakingAdapter<Chain> {
        fn wrapper() -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                abstract_cw_staking::contract::execute,
                abstract_cw_staking::contract::instantiate,
                abstract_cw_staking::contract::query,
            ))
        }
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            let mut artifacts_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            artifacts_path.push("../../../artifacts");

            ArtifactsDir::new(artifacts_path)
                .find_wasm_path("abstract_cw_staking-osmosis")
                .unwrap()
        }
    }

    impl<Chain: CwEnv> OsmosisStakingAdapter<Chain> {
        /// Stake using Abstract's OS (registered in daemon_state).
        pub fn stake(
            &self,
            stake_assets: Vec<AnsAsset>,
            provider: String,
            duration: Option<cw_utils::Duration>,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let stake_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                account_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Stake {
                        assets: stake_assets,
                        unbonding_period: duration,
                    },
                },
            });
            account
                .account
                .execute_on_module(CW_STAKING_ADAPTER_ID, stake_msg)?;
            Ok(())
        }

        pub fn unstake(
            &self,
            stake_assets: Vec<AnsAsset>,
            provider: String,
            duration: Option<cw_utils::Duration>,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let stake_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                account_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Unstake {
                        assets: stake_assets,
                        unbonding_period: duration,
                    },
                },
            });
            account
                .account
                .execute_on_module(CW_STAKING_ADAPTER_ID, stake_msg)?;
            Ok(())
        }

        pub fn claim(
            &self,
            stake_assets: Vec<AssetEntry>,
            provider: String,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let claim_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                account_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Claim {
                        assets: stake_assets,
                    },
                },
            });
            account
                .account
                .execute_on_module(CW_STAKING_ADAPTER_ID, claim_msg)?;
            Ok(())
        }

        pub fn claim_rewards(
            &self,
            stake_assets: Vec<AssetEntry>,
            provider: String,
            account: &AbstractAccount<Chain>,
        ) -> Result<(), AbstractInterfaceError> {
            let claim_rewards_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                account_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::ClaimRewards {
                        assets: stake_assets,
                    },
                },
            });
            account
                .account
                .execute_on_module(CW_STAKING_ADAPTER_ID, claim_rewards_msg)?;
            Ok(())
        }
    }

    fn setup_osmosis() -> anyhow::Result<(
        OsmosisTestTube,
        u64,
        OsmosisStakingAdapter<OsmosisTestTube>,
        AbstractAccount<OsmosisTestTube>,
    )> {
        let tube = OsmosisTestTube::new(vec![
            coin(1_000_000_000_000, ASSET_1),
            coin(1_000_000_000_000, ASSET_2),
        ]);

        let deployment = Abstract::deploy_on(tube.clone(), tube.sender_addr().to_string())?;

        let _root_os = create_default_account(&deployment.account_factory)?;
        let staking: OsmosisStakingAdapter<OsmosisTestTube> =
            OsmosisStakingAdapter::new(CW_STAKING_ADAPTER_ID, tube.clone());

        staking.deploy(CONTRACT_VERSION.parse()?, Empty {}, DeployStrategy::Error)?;

        let os = create_default_account(&deployment.account_factory)?;
        // let account_addr = os.account.address()?;
        let _account_addr = os.account.address()?;

        // transfer some LP tokens to the AbstractAccount, as if it provided liquidity
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

        // install exchange on AbstractAccount
        os.install_adapter(&staking, None)?;

        tube.bank_send(
            os.account.addr_str()?,
            coins(1_000u128, get_pool_token(pool_id)),
        )?;

        Ok((tube, pool_id, staking, os))
    }

    #[test]
    fn staking_inited() -> anyhow::Result<()> {
        let (_, pool_id, staking, _) = setup_osmosis()?;

        // query staking info
        let staking_info = staking.info(OSMOSIS.into(), vec![AssetEntry::new(LP)])?;
        let staking_coin = AssetInfoBase::native(get_pool_token(pool_id));
        assert_that!(staking_info).is_equal_to(StakingInfoResponse {
            infos: vec![StakingInfo {
                staking_target: pool_id.into(),
                staking_token: staking_coin.clone(),
                unbonding_periods: Some(vec![]),
                max_claims: None,
            }],
        });

        Ok(())
    }

    #[test]
    fn stake_lp() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let account_addr = os.account.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 stake-coins
        staking.stake(vec![AnsAsset::new(LP, 100u128)], OSMOSIS.into(), dur, &os)?;

        tube.wait_seconds(10000)?;
        // query stake
        let res = staking.staked(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
            dur,
        );

        // TODO: something needs to be version bumped for it to work
        // It's already supported on osmosis
        // assert_that!(res.unwrap_err().to_string())
        //     .contains(CwStakingError::NotImplemented("osmosis".to_owned()).to_string());

        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

        Ok(())
    }

    #[test]
    fn unstake_lp() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let account_addr = os.account.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        staking.stake(vec![AnsAsset::new(LP, 100u128)], OSMOSIS.into(), dur, &os)?;

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

        // now unbond 50
        staking.unstake(vec![AnsAsset::new(LP, 50u128)], OSMOSIS.into(), dur, &os)?;
        // query unbond
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_that!(unbonding.claims[0][0].amount).is_equal_to(Uint128::new(50));

        // Wait, and check unbonding status
        tube.wait_seconds(2)?;
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_that!(unbonding.claims[0]).is_empty();

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(50u128.to_string());
        Ok(())
    }

    #[test]
    fn claim_all() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let account_addr = os.account.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        staking.stake(vec![AnsAsset::new(LP, 100u128)], OSMOSIS.into(), dur, &os)?;

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

        // now unbond all
        staking.claim(vec![AssetEntry::new(LP)], OSMOSIS.into(), &os)?;
        // query unbond
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_that!(unbonding.claims[0][0].amount).is_equal_to(Uint128::new(100));

        // Wait, and check unbonding status
        tube.wait_seconds(2)?;
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            account_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_that!(unbonding.claims[0]).is_empty();

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: account_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins.len()).is_equal_to(0);
        Ok(())
    }

    // Currently not supported for provide/withdraw
    #[test]
    fn concentrated_liquidity() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let account_addr = os.account.address()?;

        let lp = "osmosis/osmo2,atom2";
        // transfer some LP tokens to the AbstractAccount, as if it provided liquidity
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
            .stake(vec![AnsAsset::new(lp, 100u128)], OSMOSIS.into(), dur, &os)
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
        let (mut chain, pool_id, staking, os) = setup_osmosis()?;
        // For gauge
        chain.add_balance(chain.sender_addr(), coins(1_000_000_000_000, ASSET_1))?;
        let account_addr = os.account.address()?;

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
        let time = test_tube.get_block_timestamp().plus_seconds(5);
        let lockable_durations =
            incentives.query_lockable_durations(&QueryLockableDurationsRequest {})?;
        let res = incentives.add_to_gauge(
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
        chain.wait_seconds(10);

        let res = staking.reward_tokens(OSMOSIS.to_owned(), vec![AssetEntry::new(LP)])?;
        assert_eq!(res.tokens, [[AssetInfoBase::Native(ASSET_1.to_owned())]]);
        Ok(())
    }

    #[test]
    #[ignore = "Currently broken, see https://github.com/osmosis-labs/test-tube/pull/53"]
    fn reward_tokens_create_gauge() -> anyhow::Result<()> {
        let (mut chain, pool_id, staking, os) = setup_osmosis()?;
        // For gauge
        chain.add_balance(chain.sender_addr(), coins(1_000_000_000_000, ASSET_1))?;
        let account_addr = os.account.address()?;

        let test_tube = chain.app.borrow();
        let incentives = super::incentives::Incentives::new(&*test_tube);
        let poolincentives = super::poolincentives::Poolincentives::new(&*test_tube);

        // Check that we have empty assets at start
        let res = staking.reward_tokens(OSMOSIS.to_owned(), vec![AssetEntry::new(LP)])?;
        assert_eq!(res.tokens, [[]]);

        // Now incentivize pool
        let time = test_tube.get_block_timestamp().plus_seconds(5);
        let lockable_durations =
            incentives.query_lockable_durations(&QueryLockableDurationsRequest {})?;
        let res = incentives.create_gauge(
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
        chain.wait_seconds(10);

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
