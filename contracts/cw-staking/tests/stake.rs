use abstract_boot::boot_core::{instantiate_default_mock_env, CallAs, ContractInstance, Deploy};
use abstract_boot::{Abstract, AbstractAccount, ApiDeployer};
use abstract_core::objects::{AnsAsset, AssetEntry};

use abstract_cw_staking_api::msg::{
    Claim, RewardTokensResponse, StakingInfoResponse, UnbondingResponse,
};
use boot_core::{Mock, TxHandler};
use boot_cw_plus::{Cw20ExecuteMsgFns, Cw20QueryMsgFns};
use cosmwasm_std::{coin, Addr, Empty, Uint128};
use cw_asset::AssetInfoBase;
use speculoos::*;
use wyndex_bundle::{EUR_USD_LP, WYNDEX, WYNDEX_OWNER, WYND_TOKEN};

use abstract_cw_staking_api::CW_STAKING;
use abstract_cw_staking_api::{boot::CwStakingApi, msg::CwStakingQueryMsgFns};
use common::create_default_os;

mod common;

fn setup_mock() -> anyhow::Result<(
    Mock,
    wyndex_bundle::WynDex,
    CwStakingApi<Mock>,
    AbstractAccount<Mock>,
)> {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;

    let deployment = Abstract::deploy_on(chain.clone(), "1.0.0".parse()?)?;
    let wyndex = wyndex_bundle::WynDex::store_on(chain.clone())?;

    let _root_os = create_default_os(&deployment.account_factory)?;
    let mut staking_api = CwStakingApi::new(CW_STAKING, chain.clone());

    staking_api.deploy("1.0.0".parse()?, Empty {})?;

    let os = create_default_os(&deployment.account_factory)?;
    let proxy_addr = os.proxy.address()?;
    let _manager_addr = os.manager.address()?;

    // transfer some LP tokens to the AbstractAccount, as if it provided liquidity
    wyndex
        .eur_usd_lp
        .call_as(&Addr::unchecked(WYNDEX_OWNER))
        .transfer(1000u128.into(), proxy_addr.to_string())?;

    // install exchange on AbstractAccount
    os.manager.install_module(CW_STAKING, &Empty {})?;
    // load exchange data into type
    staking_api.set_address(&Addr::unchecked(
        os.manager.module_info(CW_STAKING)?.unwrap().address,
    ));

    Ok((chain, wyndex, staking_api, os))
}

#[test]
fn staking_inited() -> anyhow::Result<()> {
    let (_, wyndex, staking_api, _) = setup_mock()?;

    // query staking info
    let staking_info = staking_api.info(WYNDEX.into(), AssetEntry::new(EUR_USD_LP))?;
    assert_that!(staking_info).is_equal_to(StakingInfoResponse {
        staking_contract_address: wyndex.eur_usd_staking,
        staking_token: AssetInfoBase::Cw20(wyndex.eur_usd_lp.address()?),
        unbonding_periods: Some(vec![
            cw_utils::Duration::Time(1),
            cw_utils::Duration::Time(2),
        ]),
        max_claims: None,
    });

    // query reward tokens
    let reward_tokens = staking_api.reward_tokens(WYNDEX.into(), AssetEntry::new(EUR_USD_LP))?;
    assert_that!(reward_tokens).is_equal_to(RewardTokensResponse {
        tokens: vec![AssetInfoBase::Native(WYND_TOKEN.to_owned())],
    });

    Ok(())
}

#[test]
fn stake_lp() -> anyhow::Result<()> {
    let (_, _, staking_api, os) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking_api.stake(AnsAsset::new(EUR_USD_LP, 100u128), WYNDEX.into(), dur)?;

    // query stake
    let staked_balance = staking_api.staked(
        WYNDEX.into(),
        proxy_addr.to_string(),
        AssetEntry::new(EUR_USD_LP),
        dur,
    )?;
    assert_that!(staked_balance.amount.u128()).is_equal_to(100u128);

    Ok(())
}

#[test]
fn unstake_lp() -> anyhow::Result<()> {
    let (_, _, staking_api, os) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking_api.stake(AnsAsset::new(EUR_USD_LP, 100u128), WYNDEX.into(), dur)?;

    // query stake
    let staked_balance = staking_api.staked(
        WYNDEX.into(),
        proxy_addr.to_string(),
        AssetEntry::new(EUR_USD_LP),
        dur,
    )?;
    assert_that!(staked_balance.amount.u128()).is_equal_to(100u128);

    // now unbond 50
    staking_api.unstake(AnsAsset::new(EUR_USD_LP, 50u128), WYNDEX.into(), dur)?;
    // query stake
    let staked_balance = staking_api.staked(
        WYNDEX.into(),
        proxy_addr.to_string(),
        AssetEntry::new(EUR_USD_LP),
        dur,
    )?;
    assert_that!(staked_balance.amount.u128()).is_equal_to(50u128);
    Ok(())
}

#[test]
fn claim_unbonded_lp() -> anyhow::Result<()> {
    let (chain, wyndex, staking_api, os) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking_api.stake(AnsAsset::new(EUR_USD_LP, 100u128), WYNDEX.into(), dur)?;

    // now unbond 50
    staking_api.unstake(AnsAsset::new(EUR_USD_LP, 50u128), WYNDEX.into(), dur)?;

    let unstake_block_info = chain.block_info()?;

    // query unbonding
    let unbonding_balance = staking_api.unbonding(
        WYNDEX.into(),
        proxy_addr.to_string(),
        AssetEntry::new(EUR_USD_LP),
    )?;
    let claimable_at = dur.unwrap().after(&unstake_block_info);
    assert_that!(unbonding_balance).is_equal_to(UnbondingResponse {
        claims: vec![Claim {
            amount: Uint128::from(50u128),
            claimable_at,
        }],
    });

    // forward 5 seconds
    chain.next_block()?;

    // now claim 50
    staking_api.claim(AssetEntry::new(EUR_USD_LP), WYNDEX.into())?;

    // query balance
    let balance = wyndex.eur_usd_lp.balance(proxy_addr.to_string())?;
    assert_that!(balance.balance.u128()).is_equal_to(950u128);

    Ok(())
}

#[test]
fn claim_rewards() -> anyhow::Result<()> {
    let (chain, mut wyndex, staking_api, os) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking_api.stake(AnsAsset::new(EUR_USD_LP, 100u128), WYNDEX.into(), dur)?;

    // forward 500 seconds
    chain.wait_blocks(100)?;

    chain.set_balance(&wyndex.eur_usd_staking, vec![coin(10_000, WYND_TOKEN)])?;
    wyndex
        .suite
        .distribute_funds(wyndex.eur_usd_staking, WYNDEX_OWNER, &[])
        .unwrap();

    // now claim rewards
    staking_api.claim_rewards(AssetEntry::new(EUR_USD_LP), WYNDEX.into())?;

    // query balance
    let balance = chain.query_balance(&proxy_addr, WYND_TOKEN)?;
    assert_that!(balance.u128()).is_equal_to(10_000u128);

    Ok(())
}
