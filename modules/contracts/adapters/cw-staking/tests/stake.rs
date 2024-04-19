mod common;

use abstract_adapter::std::{
    adapter::BaseQueryMsgFns,
    objects::{module_version::ModuleDataResponse, AnsAsset, AssetEntry},
};
use abstract_cw_staking::{
    contract::CONTRACT_VERSION, interface::CwStakingAdapter, msg::StakingQueryMsgFns,
};
use abstract_interface::{Abstract, AbstractAccount, AdapterDeployer, DeployStrategy};
use abstract_staking_standard::msg::{
    Claim, RewardTokensResponse, StakingInfo, StakingInfoResponse, UnbondingResponse,
};
use cosmwasm_std::{coin, Empty, Uint128};
use cw20::msg::Cw20ExecuteMsgFns;
use cw20_base::msg::QueryMsgFns;
use cw_asset::AssetInfoBase;
use cw_orch::prelude::*;
use speculoos::*;
use wyndex_bundle::{EUR_USD_LP, WYNDEX as WYNDEX_WITHOUT_CHAIN, WYNDEX_OWNER, WYND_TOKEN};

const WYNDEX: &str = "cosmos-testnet>wyndex";

use abstract_cw_staking::CW_STAKING_ADAPTER_ID;
use common::create_default_account;

fn setup_mock() -> anyhow::Result<(
    MockBech32,
    wyndex_bundle::WynDex,
    CwStakingAdapter<MockBech32>,
    AbstractAccount<MockBech32>,
)> {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();

    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let wyndex = wyndex_bundle::WynDex::store_on(chain.clone())?;

    let _root_os = create_default_account(&deployment.account_factory)?;
    let staking = CwStakingAdapter::new(CW_STAKING_ADAPTER_ID, chain.clone());

    staking.deploy(CONTRACT_VERSION.parse()?, Empty {}, DeployStrategy::Try)?;

    let os = create_default_account(&deployment.account_factory)?;
    let proxy_addr = os.proxy.address()?;
    let _manager_addr = os.manager.address()?;

    // transfer some LP tokens to the AbstractAccount, as if it provided liquidity
    wyndex
        .eur_usd_lp
        .call_as(&chain.addr_make(WYNDEX_OWNER))
        .transfer(1000u128.into(), proxy_addr.to_string())?;

    // install exchange on AbstractAccount
    os.install_adapter(&staking, None)?;

    Ok((chain, wyndex, staking, os))
}

#[test]
fn staking_inited() -> anyhow::Result<()> {
    let (_, wyndex, staking, _) = setup_mock()?;

    // query staking info
    let staking_info = staking.info(WYNDEX.into(), vec![AssetEntry::new(EUR_USD_LP)])?;
    assert_that!(staking_info).is_equal_to(StakingInfoResponse {
        infos: vec![StakingInfo {
            staking_target: wyndex.eur_usd_staking.into(),
            staking_token: AssetInfoBase::Cw20(wyndex.eur_usd_lp.address()?),
            unbonding_periods: Some(vec![
                cw_utils::Duration::Time(1),
                cw_utils::Duration::Time(2),
            ]),
            max_claims: None,
        }],
    });

    // query reward tokens
    let reward_tokens = staking.reward_tokens(WYNDEX.into(), vec![AssetEntry::new(EUR_USD_LP)])?;
    assert_that!(reward_tokens).is_equal_to(RewardTokensResponse {
        tokens: vec![vec![AssetInfoBase::Native(WYND_TOKEN.to_owned())]],
    });

    let module_data = staking.module_data()?;
    assert_eq!(
        module_data,
        ModuleDataResponse {
            module_id: CW_STAKING_ADAPTER_ID.to_owned(),
            version: CONTRACT_VERSION.to_owned(),
            dependencies: vec![],
            metadata: None
        }
    );
    Ok(())
}

#[test]
fn stake_lp() -> anyhow::Result<()> {
    let (_, _, staking, os) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking.stake(AnsAsset::new(EUR_USD_LP, 100u128), WYNDEX.into(), dur, &os)?;

    // query stake
    let staked_balance = staking.staked(
        WYNDEX.into(),
        proxy_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
        dur,
    )?;
    assert_that!(staked_balance.amounts[0].u128()).is_equal_to(100u128);

    Ok(())
}

#[test]
fn stake_lp_wthout_chain() -> anyhow::Result<()> {
    let (_, _, staking, os) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking.stake(
        AnsAsset::new(EUR_USD_LP, 100u128),
        WYNDEX_WITHOUT_CHAIN.into(),
        dur,
        &os,
    )?;

    // query stake
    let staked_balance = staking.staked(
        WYNDEX.into(),
        proxy_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
        dur,
    )?;
    assert_that!(staked_balance.amounts[0].u128()).is_equal_to(100u128);

    Ok(())
}

#[test]
fn unstake_lp() -> anyhow::Result<()> {
    let (_, _, staking, os) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking.stake(AnsAsset::new(EUR_USD_LP, 100u128), WYNDEX.into(), dur, &os)?;

    // query stake
    let staked_balance = staking.staked(
        WYNDEX.into(),
        proxy_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
        dur,
    )?;
    assert_that!(staked_balance.amounts[0].u128()).is_equal_to(100u128);

    // now unbond 50
    staking.unstake(AnsAsset::new(EUR_USD_LP, 50u128), WYNDEX.into(), dur, &os)?;
    // query stake
    let staked_balance = staking.staked(
        WYNDEX.into(),
        proxy_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
        dur,
    )?;
    assert_that!(staked_balance.amounts[0].u128()).is_equal_to(50u128);
    Ok(())
}

#[test]
fn claim_unbonded_lp() -> anyhow::Result<()> {
    let (chain, wyndex, staking, os) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let dur = cw_utils::Duration::Time(2);

    // stake 100 EUR
    staking.stake(
        AnsAsset::new(EUR_USD_LP, 100u128),
        WYNDEX.into(),
        Some(dur),
        &os,
    )?;

    // now unbond 50
    staking.unstake(
        AnsAsset::new(EUR_USD_LP, 50u128),
        WYNDEX.into(),
        Some(dur),
        &os,
    )?;

    let unstake_block_info = chain.block_info()?;

    // query unbonding
    let unbonding_balance = staking.unbonding(
        WYNDEX.into(),
        proxy_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
    )?;
    let claimable_at = dur.after(&unstake_block_info);
    assert_that!(unbonding_balance).is_equal_to(UnbondingResponse {
        claims: vec![vec![Claim {
            amount: Uint128::from(50u128),
            claimable_at,
        }]],
    });

    // forward 5 seconds
    chain.next_block()?;

    // now claim 50
    staking.claim(AssetEntry::new(EUR_USD_LP), WYNDEX.into(), &os)?;

    // query balance
    let balance = wyndex.eur_usd_lp.balance(proxy_addr.to_string())?;
    assert_that!(balance.balance.u128()).is_equal_to(950u128);

    Ok(())
}

#[test]
fn claim_rewards() -> anyhow::Result<()> {
    let (chain, mut wyndex, staking, os) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking.stake(AnsAsset::new(EUR_USD_LP, 100u128), WYNDEX.into(), dur, &os)?;

    // forward 500 seconds
    chain.wait_blocks(100)?;

    chain.set_balance(&wyndex.eur_usd_staking, vec![coin(10_000, WYND_TOKEN)])?;
    wyndex
        .suite
        .distribute_funds(wyndex.eur_usd_staking, &chain.addr_make(WYNDEX_OWNER), &[])
        .unwrap();

    // now claim rewards
    staking.claim_rewards(AssetEntry::new(EUR_USD_LP), WYNDEX.into(), &os)?;

    // query balance
    let balance = chain.query_balance(&proxy_addr, WYND_TOKEN)?;
    assert_that!(balance.u128()).is_equal_to(10_000u128);

    Ok(())
}
