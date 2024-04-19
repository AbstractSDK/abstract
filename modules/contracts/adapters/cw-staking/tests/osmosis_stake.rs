#![allow(unused)]
mod common;

#[cfg(feature = "osmosis-test")]
mod osmosis_test {

    use std::path::PathBuf;

    use abstract_cw_staking::{
        contract::CONTRACT_VERSION,
        msg::{
            ExecuteMsg, InstantiateMsg, QueryMsg, RewardTokensResponse, StakingAction,
            StakingExecuteMsg, StakingQueryMsgFns,
        },
    };
    use abstract_interface::{
        Abstract, AbstractAccount, AbstractInterfaceError, AdapterDeployer, DeployStrategy, Manager,
    };
    use abstract_staking_standard::{
        msg::{StakingInfo, StakingInfoResponse},
        CwStakingError,
    };
    use abstract_std::{
        adapter,
        ans_host::ExecuteMsgFns,
        objects::{
            pool_id::PoolAddressBase, AccountId, AnsAsset, AssetEntry, PoolMetadata, PoolType,
        },
        MANAGER,
    };
    use cosmwasm_std::{coin, coins, Addr, Empty, Uint128};
    use cw_asset::AssetInfoBase;
    use cw_orch::{
        interface,
        osmosis_test_tube::osmosis_test_tube::{
            osmosis_std::types::osmosis::lockup::{
                AccountLockedCoinsRequest, AccountLockedCoinsResponse,
            },
            Runner,
        },
        prelude::*,
    };
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

    impl<Chain: CwEnv> Uploadable for OsmosisStakingAdapter<Chain> {
        fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                abstract_cw_staking::contract::execute,
                abstract_cw_staking::contract::instantiate,
                abstract_cw_staking::contract::query,
            ))
        }
        fn wasm(&self) -> WasmPath {
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
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Stake {
                        assets: stake_assets,
                        unbonding_period: duration,
                    },
                },
            });
            account
                .manager
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
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Unstake {
                        assets: stake_assets,
                        unbonding_period: duration,
                    },
                },
            });
            account
                .manager
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
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::Claim {
                        assets: stake_assets,
                    },
                },
            });
            account
                .manager
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
                proxy_address: None,
                request: StakingExecuteMsg {
                    provider,
                    action: StakingAction::ClaimRewards {
                        assets: stake_assets,
                    },
                },
            });
            account
                .manager
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

        let deployment = Abstract::deploy_on(tube.clone(), tube.sender().to_string())?;

        let _root_os = create_default_account(&deployment.account_factory)?;
        let staking: OsmosisStakingAdapter<OsmosisTestTube> =
            OsmosisStakingAdapter::new(CW_STAKING_ADAPTER_ID, tube.clone());

        staking.deploy(CONTRACT_VERSION.parse()?, Empty {}, DeployStrategy::Error)?;

        let os = create_default_account(&deployment.account_factory)?;
        // let proxy_addr = os.proxy.address()?;
        let _manager_addr = os.manager.address()?;

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
            os.proxy.addr_str()?,
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

        // query reward tokens should be empty for osmosis
        let reward_tokens: RewardTokensResponse =
            staking.reward_tokens(OSMOSIS.into(), vec![AssetEntry::new(LP)])?;

        assert_eq!(reward_tokens, RewardTokensResponse { tokens: vec![] });
        Ok(())
    }

    #[test]
    fn stake_lp() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let proxy_addr = os.proxy.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 stake-coins
        staking.stake(vec![AnsAsset::new(LP, 100u128)], OSMOSIS.into(), dur, &os)?;

        tube.wait_seconds(10000)?;
        // query stake
        let res = staking.staked(
            OSMOSIS.into(),
            proxy_addr.to_string(),
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
                owner: proxy_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

        Ok(())
    }

    #[test]
    fn unstake_lp() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let proxy_addr = os.proxy.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        staking.stake(vec![AnsAsset::new(LP, 100u128)], OSMOSIS.into(), dur, &os)?;

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: proxy_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

        // now unbond 50
        staking.unstake(vec![AnsAsset::new(LP, 50u128)], OSMOSIS.into(), dur, &os)?;
        // query unbond
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            proxy_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_that!(unbonding.claims[0][0].amount).is_equal_to(Uint128::new(50));

        // Wait, and check unbonding status
        tube.wait_seconds(2)?;
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            proxy_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_that!(unbonding.claims[0]).is_empty();

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: proxy_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(50u128.to_string());
        Ok(())
    }

    #[test]
    fn claim_all() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let proxy_addr = os.proxy.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        staking.stake(vec![AnsAsset::new(LP, 100u128)], OSMOSIS.into(), dur, &os)?;

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: proxy_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

        // now unbond all
        staking.claim(vec![AssetEntry::new(LP)], OSMOSIS.into(), &os)?;
        // query unbond
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            proxy_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_that!(unbonding.claims[0][0].amount).is_equal_to(Uint128::new(100));

        // Wait, and check unbonding status
        tube.wait_seconds(2)?;
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            proxy_addr.to_string(),
            vec![AssetEntry::new(LP)],
        )?;
        assert_that!(unbonding.claims[0]).is_empty();

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: proxy_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins.len()).is_equal_to(0);
        Ok(())
    }

    // Currently not supported for provide/withdraw
    #[test]
    fn concentrated_liquidity() -> anyhow::Result<()> {
        let (tube, _, staking, os) = setup_osmosis()?;
        let proxy_addr = os.proxy.address()?;

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
}
