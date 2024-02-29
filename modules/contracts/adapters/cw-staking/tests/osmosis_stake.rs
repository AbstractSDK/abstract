#![cfg(feature = "osmosis-test")]
mod common;

mod osmosis_test {
    use abstract_core::objects::LpToken;
    use osmosis_pool::helpers::osmosis_pool_token;
    use osmosis_pool::OsmosisPools;
    use osmosis_pool::INCENTIVES_AMOUNT;
    use osmosis_pool::NUM_EPOCHS_POOL;
    use osmosis_pool::OSMOSIS;
    use std::path::PathBuf;

    use abstract_core::{
        adapter,
        objects::{AnsAsset, AssetEntry, PoolType},
    };
    use abstract_cw_staking::{
        contract::CONTRACT_VERSION,
        msg::{
            ExecuteMsg, InstantiateMsg, QueryMsg, StakingAction, StakingExecuteMsg,
            StakingQueryMsgFns,
        },
    };
    use abstract_interface::{
        Abstract, AbstractAccount, AbstractInterfaceError, AdapterDeployer, DeployStrategy,
    };
    use abstract_staking_standard::{
        msg::{StakingInfo, StakingInfoResponse},
        CwStakingError,
    };
    use cosmwasm_std::{coin, coins, Empty, Uint128};
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

    use abstract_cw_staking::CW_STAKING_ADAPTER_ID;

    use crate::common::create_default_account;

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
        OsmosisPools,
        OsmosisStakingAdapter<OsmosisTestTube>,
        AbstractAccount<OsmosisTestTube>,
    )> {
        let mut chain = OsmosisTestTube::new(vec![coin(1_000_000_000_000, "uosmo")]);

        let deployment = Abstract::deploy_on(chain.clone(), chain.sender().to_string())?;
        let osmosis_pools = OsmosisPools::deploy_on(chain.clone(), Empty {})?;

        let staking: OsmosisStakingAdapter<OsmosisTestTube> =
            OsmosisStakingAdapter::new(CW_STAKING_ADAPTER_ID, chain.clone());

        staking.deploy(CONTRACT_VERSION.parse()?, Empty {}, DeployStrategy::Error)?;

        let os = create_default_account(&deployment.account_factory)?;
        let proxy_addr = os.proxy.address()?;
        chain.add_balance(
            &proxy_addr,
            coins(100, osmosis_pool_token(osmosis_pools.eur_usd_pool)),
        )?;
        chain.add_balance(
            &proxy_addr,
            coins(
                100,
                osmosis_pool_token(osmosis_pools.fast_incentivized_eur_usd_pool),
            ),
        )?;

        // install exchange on AbstractAccount
        os.install_adapter(&staking, None)?;

        Ok((chain, osmosis_pools, staking, os))
    }

    #[test]
    fn staking_inited() -> anyhow::Result<()> {
        let (_, osmosis, staking, _) = setup_osmosis()?;

        // query staking info
        let lp = LpToken::new(OSMOSIS, vec![osmosis.eur_token, osmosis.usd_token]).to_string();
        let staking_info = staking.info(OSMOSIS.into(), vec![AssetEntry::new(&lp)])?;
        let staking_coin = AssetInfoBase::native(osmosis_pool_token(osmosis.eur_usd_pool));
        assert_that!(staking_info).is_equal_to(StakingInfoResponse {
            infos: vec![StakingInfo {
                staking_target: osmosis.eur_usd_pool.into(),
                staking_token: staking_coin.clone(),
                unbonding_periods: Some(vec![]),
                max_claims: None,
            }],
        });

        // query reward tokens
        let res: CwOrchError = staking
            .reward_tokens(OSMOSIS.into(), vec![AssetEntry::new(&lp)])
            .unwrap_err();
        assert_that!(res.to_string())
            .contains(CwStakingError::NotImplemented("osmosis".to_owned()).to_string());

        Ok(())
    }

    #[test]
    fn stake_lp() -> anyhow::Result<()> {
        let (tube, osmosis, staking, os) = setup_osmosis()?;
        let proxy_addr = os.proxy.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 stake-coins
        let lp = LpToken::new(OSMOSIS, vec![osmosis.eur_token, osmosis.usd_token]).to_string();
        staking.stake(
            vec![AnsAsset::new(lp.clone(), 100u128)],
            OSMOSIS.into(),
            dur,
            &os,
        )?;

        tube.wait_seconds(10000)?;
        // query stake
        let res = staking.staked(
            OSMOSIS.into(),
            proxy_addr.to_string(),
            vec![AssetEntry::new(&lp)],
            dur,
        )?;

        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: proxy_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(res.amounts[0].to_string());
        assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

        Ok(())
    }

    #[test]
    fn unstake_lp() -> anyhow::Result<()> {
        let (tube, osmosis, staking, os) = setup_osmosis()?;
        let proxy_addr = os.proxy.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        let lp = LpToken::new(OSMOSIS, vec![osmosis.eur_token, osmosis.usd_token]).to_string();
        staking.stake(
            vec![AnsAsset::new(lp.clone(), 100u128)],
            OSMOSIS.into(),
            dur,
            &os,
        )?;

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: proxy_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

        // now unbond 50
        staking.unstake(
            vec![AnsAsset::new(lp.clone(), 50u128)],
            OSMOSIS.into(),
            dur,
            &os,
        )?;
        // query unbond
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            proxy_addr.to_string(),
            vec![AssetEntry::new(&lp)],
        )?;
        assert_that!(unbonding.claims[0][0].amount).is_equal_to(Uint128::new(50));

        // Wait, and check unbonding status
        tube.wait_seconds(2)?;
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            proxy_addr.to_string(),
            vec![AssetEntry::new(&lp)],
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
        let (tube, osmosis, staking, os) = setup_osmosis()?;
        let proxy_addr = os.proxy.address()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        let lp = LpToken::new(OSMOSIS, vec![osmosis.eur_token_fast, osmosis.usd_token]).to_string();
        staking.stake(vec![AnsAsset::new(&lp, 100u128)], OSMOSIS.into(), dur, &os)?;

        // query stake
        let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
            "/osmosis.lockup.Query/AccountLockedCoins",
            &AccountLockedCoinsRequest {
                owner: proxy_addr.to_string(),
            },
        )?;
        assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());
        let pool_balance = tube.bank_querier().balance(
            &proxy_addr,
            Some(osmosis_pool_token(osmosis.fast_incentivized_eur_usd_pool)),
        )?;
        assert_that!(pool_balance[0].amount.u128()).is_equal_to(0);

        // now unbond all
        staking.claim(vec![AssetEntry::new(&lp)], OSMOSIS.into(), &os)?;
        // query unbond
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            proxy_addr.to_string(),
            vec![AssetEntry::new(&lp)],
        )?;
        assert_that!(unbonding.claims[0][0].amount).is_equal_to(Uint128::new(100));

        // Wait, and check unbonding status
        tube.wait_seconds(2)?;
        let unbonding = staking.unbonding(
            OSMOSIS.into(),
            proxy_addr.to_string(),
            vec![AssetEntry::new(&lp)],
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

        let pool_balance = tube.bank_querier().balance(
            &proxy_addr,
            Some(osmosis_pool_token(osmosis.fast_incentivized_eur_usd_pool)),
        )?;
        assert_that!(pool_balance[0].amount.u128()).is_equal_to(100);

        let incentives = tube
            .bank_querier()
            .balance(proxy_addr, Some(osmosis.fast_incentives_token))?;
        assert_that!(incentives[0].amount.u128())
            .is_equal_to(INCENTIVES_AMOUNT / NUM_EPOCHS_POOL as u128 * 2);

        Ok(())
    }

    #[test]
    fn claim_rewards_fails() -> anyhow::Result<()> {
        let (_tube, osmosis, staking, os) = setup_osmosis()?;

        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        let lp = LpToken::new(OSMOSIS, vec![osmosis.eur_token_fast, osmosis.usd_token]).to_string();
        staking.stake(vec![AnsAsset::new(&lp, 100u128)], OSMOSIS.into(), dur, &os)?;

        // now claim rewards and fail
        let err = staking
            .claim_rewards(vec![AssetEntry::new(&lp)], OSMOSIS.into(), &os)
            .unwrap_err();
        if let AbstractInterfaceError::Orch(CwOrchError::StdErr(e)) = err {
            let expected_err = CwStakingError::NotImplemented(
                "osmosis does not support claiming rewards".to_owned(),
            );
            assert!(e.contains(&expected_err.to_string()));
        } else {
            panic!("Expected stderror");
        };
        Ok(())
    }

    // Currently not supported for provide/withdraw and not for stake either
    #[test]
    fn concentrated_liquidity() -> anyhow::Result<()> {
        let (mut tube, mut osmosis, staking, os) = setup_osmosis()?;
        let proxy_addr = os.proxy.address()?;

        let pool_id = osmosis.suite.create_concentrated_liquidity_pool(
            coin(1_000, osmosis.eur_token),
            coin(1_000, osmosis.usd_token),
            Some("eur2"),
            Some("usd2"),
        )?;
        let dur = Some(cw_utils::Duration::Time(2));

        // stake 100 EUR
        let lp = LpToken::new(OSMOSIS, vec!["eur2", "usd2"]);
        tube.add_balance(proxy_addr, coins(100u128, osmosis_pool_token(pool_id)))?;
        let err = staking
            .stake(
                vec![AnsAsset::new(lp.to_string(), 100u128)],
                OSMOSIS.into(),
                dur,
                &os,
            )
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
