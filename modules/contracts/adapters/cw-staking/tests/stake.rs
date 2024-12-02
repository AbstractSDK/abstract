use abstract_adapter::abstract_interface::{Abstract, AccountI, AdapterDeployer, DeployStrategy};
use abstract_adapter::std::{
    adapter::BaseQueryMsgFns,
    objects::{module_version::ModuleDataResponse, AnsAsset, AssetEntry},
};
use abstract_client::builder::cw20_builder::{ExecuteMsgInterfaceFns, QueryMsgInterfaceFns};
use abstract_cw_staking::{
    contract::CONTRACT_VERSION, interface::CwStakingAdapter, msg::StakingQueryMsgFns,
};
use abstract_staking_standard::msg::{
    Claim, RewardTokensResponse, StakingInfo, StakingInfoResponse, UnbondingResponse,
};
use cosmwasm_std::{coin, Uint128};
use cw_asset::AssetInfoBase;
use cw_orch::prelude::*;
use mockdex_bundle::{EUR_USD_LP, WYNDEX as WYNDEX_WITHOUT_CHAIN, WYNDEX_OWNER, WYND_TOKEN};

const WYNDEX: &str = "cosmos-testnet>wyndex";

use abstract_cw_staking::CW_STAKING_ADAPTER_ID;
use abstract_integration_tests::create_default_account;

fn setup_mock() -> anyhow::Result<(
    MockBech32,
    mockdex_bundle::WynDex,
    CwStakingAdapter<MockBech32>,
    AccountI<MockBech32>,
)> {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();

    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let wyndex = mockdex_bundle::WynDex::store_on(chain.clone())?;

    let _root_os = create_default_account(&sender, &deployment)?;
    let staking = CwStakingAdapter::new(CW_STAKING_ADAPTER_ID, chain.clone());

    staking.deploy(CONTRACT_VERSION.parse()?, Empty {}, DeployStrategy::Try)?;

    let account = create_default_account(&sender, &deployment)?;
    let account_addr = account.address()?;

    // transfer some LP tokens to the AccountI, as if it provided liquidity
    wyndex
        .eur_usd_lp
        .call_as(&chain.addr_make(WYNDEX_OWNER))
        .transfer(1000u128, account_addr.to_string())?;

    // install exchange on AccountI
    account.install_adapter(&staking, &[])?;

    Ok((chain, wyndex, staking, account))
}

#[test]
fn staking_inited() -> anyhow::Result<()> {
    let (_, wyndex, staking, _) = setup_mock()?;

    // query staking info
    let staking_info = staking.info(WYNDEX.into(), vec![AssetEntry::new(EUR_USD_LP)])?;
    assert_eq!(
        staking_info,
        StakingInfoResponse {
            infos: vec![StakingInfo {
                staking_target: wyndex.eur_usd_staking.into(),
                staking_token: AssetInfoBase::Cw20(wyndex.eur_usd_lp.address()?),
                unbonding_periods: Some(vec![
                    cw_utils::Duration::Time(1),
                    cw_utils::Duration::Time(2),
                ]),
                max_claims: None,
            }],
        }
    );

    // query reward tokens
    let reward_tokens = staking.reward_tokens(WYNDEX.into(), vec![AssetEntry::new(EUR_USD_LP)])?;
    assert_eq!(
        reward_tokens,
        RewardTokensResponse {
            tokens: vec![vec![AssetInfoBase::Native(WYND_TOKEN.to_owned())]],
        }
    );

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
    let (_, _, staking, account) = setup_mock()?;
    let account_addr = account.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking.stake(
        AnsAsset::new(EUR_USD_LP, 100u128),
        WYNDEX.into(),
        dur,
        &account,
    )?;

    // query stake
    let staked_balance = staking.staked(
        WYNDEX.into(),
        account_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
        dur,
    )?;
    assert_eq!(staked_balance.amounts[0].u128(), 100u128);

    Ok(())
}

#[test]
fn stake_lp_wthout_chain() -> anyhow::Result<()> {
    let (_, _, staking, account) = setup_mock()?;
    let account_addr = account.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking.stake(
        AnsAsset::new(EUR_USD_LP, 100u128),
        WYNDEX_WITHOUT_CHAIN.into(),
        dur,
        &account,
    )?;

    // query stake
    let staked_balance = staking.staked(
        WYNDEX.into(),
        account_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
        dur,
    )?;
    assert_eq!(staked_balance.amounts[0].u128(), 100u128);

    Ok(())
}

#[test]
fn unstake_lp() -> anyhow::Result<()> {
    let (_, _, staking, account) = setup_mock()?;
    let account_addr = account.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking.stake(
        AnsAsset::new(EUR_USD_LP, 100u128),
        WYNDEX.into(),
        dur,
        &account,
    )?;

    // query stake
    let staked_balance = staking.staked(
        WYNDEX.into(),
        account_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
        dur,
    )?;
    assert_eq!(staked_balance.amounts[0].u128(), 100u128);

    // now unbond 50
    staking.unstake(
        AnsAsset::new(EUR_USD_LP, 50u128),
        WYNDEX.into(),
        dur,
        &account,
    )?;
    // query stake
    let staked_balance = staking.staked(
        WYNDEX.into(),
        account_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
        dur,
    )?;
    assert_eq!(staked_balance.amounts[0].u128(), 50u128);
    Ok(())
}

#[test]
fn claim_unbonded_lp() -> anyhow::Result<()> {
    let (chain, wyndex, staking, account) = setup_mock()?;
    let account_addr = account.address()?;

    let dur = cw_utils::Duration::Time(2);

    // stake 100 EUR
    staking.stake(
        AnsAsset::new(EUR_USD_LP, 100u128),
        WYNDEX.into(),
        Some(dur),
        &account,
    )?;

    // now unbond 50
    staking.unstake(
        AnsAsset::new(EUR_USD_LP, 50u128),
        WYNDEX.into(),
        Some(dur),
        &account,
    )?;

    let unstake_block_info = chain.block_info()?;

    // query unbonding
    let unbonding_balance = staking.unbonding(
        WYNDEX.into(),
        account_addr.to_string(),
        vec![AssetEntry::new(EUR_USD_LP)],
    )?;
    let claimable_at = dur.after(&unstake_block_info);
    assert_eq!(
        unbonding_balance,
        UnbondingResponse {
            claims: vec![vec![Claim {
                amount: Uint128::from(50u128),
                claimable_at,
            }]],
        }
    );

    // forward 5 seconds
    chain.next_block()?;

    // now claim 50
    staking.claim(AssetEntry::new(EUR_USD_LP), WYNDEX.into(), &account)?;

    // query balance
    let balance = wyndex.eur_usd_lp.balance(account_addr.to_string())?;
    assert_eq!(balance.balance.u128(), 950u128);

    Ok(())
}

#[test]
fn claim_rewards() -> anyhow::Result<()> {
    let (chain, mut wyndex, staking, account) = setup_mock()?;
    let account_addr = account.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 EUR
    staking.stake(
        AnsAsset::new(EUR_USD_LP, 100u128),
        WYNDEX.into(),
        dur,
        &account,
    )?;

    // forward 500 seconds
    chain.wait_blocks(100)?;

    chain.set_balance(&wyndex.eur_usd_staking, vec![coin(10_000, WYND_TOKEN)])?;
    wyndex
        .suite
        .distribute_funds(wyndex.eur_usd_staking, &chain.addr_make(WYNDEX_OWNER), &[])
        .unwrap();

    // now claim rewards
    staking.claim_rewards(AssetEntry::new(EUR_USD_LP), WYNDEX.into(), &account)?;

    // query balance
    let balance = chain.query_balance(&account_addr, WYND_TOKEN)?;
    assert_eq!(balance.u128(), 10_000u128);

    Ok(())
}
