mod common;

use abstract_core::adapter;
use abstract_core::ans_host::ExecuteMsgFns;
use abstract_core::objects::pool_id::PoolAddressBase;
use abstract_core::objects::PoolMetadata;
use abstract_core::MANAGER;
use abstract_cw_staking::contract::CONTRACT_VERSION;
use abstract_cw_staking::msg::StakingQueryMsgFns;
use abstract_interface::Abstract;
use abstract_interface::AbstractAccount;
use abstract_interface::AbstractInterfaceError;
use abstract_interface::AdapterDeployer;
use abstract_interface::Manager;
use abstract_staking_adapter_traits::CwStakingError;
use cosmwasm_std::coins;

use abstract_cw_staking::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, StakingAction, StakingExecuteMsg,
};

use abstract_core::objects::{AnsAsset, AssetEntry};
use cw_orch::deploy::Deploy;

use abstract_staking_adapter_traits::msg::StakingInfoResponse;
use cosmwasm_std::{coin, Addr, Empty, Uint128};
use cw_asset::AssetInfoBase;
use cw_orch::interface;
use cw_orch::osmosis_test_tube::osmosis_test_tube::osmosis_std::types::osmosis::lockup::AccountLockedCoinsRequest;
use cw_orch::osmosis_test_tube::osmosis_test_tube::osmosis_std::types::osmosis::lockup::AccountLockedCoinsResponse;
use cw_orch::osmosis_test_tube::osmosis_test_tube::Runner;
use cw_orch::prelude::*;
use speculoos::*;

const OSMOSIS: &str = "osmosis";
const DENOM: &str = "uosmo";

const ASSET_1: &str = DENOM;
const ASSET_2: &str = "uatom";

pub const LP: &str = "osmosis/osmo,atom";

use abstract_cw_staking::CW_STAKING;
use common::create_default_account;
use speculoos::string::StrAssertions;
use speculoos::vec::VecAssertions;

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
        artifacts_dir_from_workspace!()
            .find_wasm_path("abstract_cw_staking-osmosis")
            .unwrap()
    }
}

impl<Chain: CwEnv> OsmosisStakingAdapter<Chain> {
    /// Swap using Abstract's OS (registered in daemon_state).
    pub fn stake(
        &self,
        stake_asset: AnsAsset,
        provider: String,
        duration: Option<cw_utils::Duration>,
    ) -> Result<(), AbstractInterfaceError> {
        let manager = Manager::new(MANAGER, self.get_chain().clone());
        let stake_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
            proxy_address: None,
            request: StakingExecuteMsg {
                provider,
                action: StakingAction::Stake {
                    asset: stake_asset,
                    unbonding_period: duration,
                },
            },
        });
        manager.execute_on_module(CW_STAKING, stake_msg)?;
        Ok(())
    }

    pub fn unstake(
        &self,
        stake_asset: AnsAsset,
        provider: String,
        duration: Option<cw_utils::Duration>,
    ) -> Result<(), AbstractInterfaceError> {
        let manager = Manager::new(MANAGER, self.get_chain().clone());
        let stake_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
            proxy_address: None,
            request: StakingExecuteMsg {
                provider,
                action: StakingAction::Unstake {
                    asset: stake_asset,
                    unbonding_period: duration,
                },
            },
        });
        manager.execute_on_module(CW_STAKING, stake_msg)?;
        Ok(())
    }

    pub fn claim(
        &self,
        stake_asset: AssetEntry,
        provider: String,
    ) -> Result<(), AbstractInterfaceError> {
        let manager = Manager::new(MANAGER, self.get_chain().clone());
        let claim_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
            proxy_address: None,
            request: StakingExecuteMsg {
                provider,
                action: StakingAction::Claim { asset: stake_asset },
            },
        });
        manager.execute_on_module(CW_STAKING, claim_msg)?;
        Ok(())
    }

    pub fn claim_rewards(
        &self,
        stake_asset: AssetEntry,
        provider: String,
    ) -> Result<(), AbstractInterfaceError> {
        let manager = Manager::new(MANAGER, self.get_chain().clone());
        let claim_rewards_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
            proxy_address: None,
            request: StakingExecuteMsg {
                provider,
                action: StakingAction::ClaimRewards { asset: stake_asset },
            },
        });
        manager.execute_on_module(CW_STAKING, claim_rewards_msg)?;
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

    let deployment = Abstract::deploy_on(tube.clone(), Empty{})?;

    let _root_os = create_default_account(&deployment.account_factory)?;
    let staking: OsmosisStakingAdapter<OsmosisTestTube> =
        OsmosisStakingAdapter::new(CW_STAKING, tube.clone());

    staking.deploy(CONTRACT_VERSION.parse()?, Empty {})?;

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
    os.manager.install_module(CW_STAKING, &Empty {}, None)?;
    // load exchange data into type
    staking.set_address(&Addr::unchecked(
        os.manager.module_info(CW_STAKING)?.unwrap().address,
    ));

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
    let staking_info = staking.info(OSMOSIS.into(), AssetEntry::new(LP))?;
    let staking_coin = AssetInfoBase::native(get_pool_token(pool_id));
    assert_that!(staking_info).is_equal_to(StakingInfoResponse {
        staking_target: pool_id.into(),
        staking_token: staking_coin.clone(),
        unbonding_periods: Some(vec![]),
        max_claims: None,
    });

    // query reward tokens
    let res: CwOrchError = staking
        .reward_tokens(OSMOSIS.into(), AssetEntry::new(LP))
        .unwrap_err();
    assert_that!(res.to_string())
        .contains(CwStakingError::NotImplemented("osmosis".to_owned()).to_string());

    Ok(())
}

#[test]
fn stake_lp() -> anyhow::Result<()> {
    let (tube, _, staking, os) = setup_osmosis()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 stake-coins
    staking.stake(AnsAsset::new(LP, 100u128), OSMOSIS.into(), dur)?;

    tube.wait_seconds(10000)?;
    // query stake
    let res = staking.staked(
        OSMOSIS.into(),
        proxy_addr.to_string(),
        AssetEntry::new(LP),
        dur,
    );

    assert_that!(res.unwrap_err().to_string())
        .contains(CwStakingError::NotImplemented("osmosis".to_owned()).to_string());

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
    staking.stake(AnsAsset::new(LP, 100u128), OSMOSIS.into(), dur)?;

    // query stake
    let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
        "/osmosis.lockup.Query/AccountLockedCoins",
        &AccountLockedCoinsRequest {
            owner: proxy_addr.to_string(),
        },
    )?;
    assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

    // now unbond 50
    staking.unstake(AnsAsset::new(LP, 50u128), OSMOSIS.into(), dur)?;
    // query unbond
    let unbonding =
        staking.unbonding(OSMOSIS.into(), proxy_addr.to_string(), AssetEntry::new(LP))?;
    assert_that!(unbonding.claims[0].amount).is_equal_to(Uint128::new(50));

    // Wait, and check unbonding status
    tube.wait_seconds(2)?;
    let unbonding =
        staking.unbonding(OSMOSIS.into(), proxy_addr.to_string(), AssetEntry::new(LP))?;
    assert_that!(unbonding.claims).is_empty();

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
    staking.stake(AnsAsset::new(LP, 100u128), OSMOSIS.into(), dur)?;

    // query stake
    let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
        "/osmosis.lockup.Query/AccountLockedCoins",
        &AccountLockedCoinsRequest {
            owner: proxy_addr.to_string(),
        },
    )?;
    assert_that!(staked_balance.coins[0].amount).is_equal_to(100u128.to_string());

    // now unbond all
    staking.claim(AssetEntry::new(LP), OSMOSIS.into())?;
    // query unbond
    let unbonding =
        staking.unbonding(OSMOSIS.into(), proxy_addr.to_string(), AssetEntry::new(LP))?;
    assert_that!(unbonding.claims[0].amount).is_equal_to(Uint128::new(100));

    // Wait, and check unbonding status
    tube.wait_seconds(2)?;
    let unbonding =
        staking.unbonding(OSMOSIS.into(), proxy_addr.to_string(), AssetEntry::new(LP))?;
    assert_that!(unbonding.claims).is_empty();

    // query stake
    let staked_balance: AccountLockedCoinsResponse = tube.app.borrow().query(
        "/osmosis.lockup.Query/AccountLockedCoins",
        &AccountLockedCoinsRequest {
            owner: proxy_addr.to_string(),
        },
    )?;
    assert_that!(staked_balance.coins).is_empty();
    Ok(())
}
