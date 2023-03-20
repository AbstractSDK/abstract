mod common;

use abstract_boot::{Abstract, ApiDeployer, OS};
use abstract_os::objects::{AnsAsset, AssetEntry};
use abstract_testing::ROOT_USER;
use boot_core::{instantiate_default_mock_env, ContractInstance};
use boot_core::{BootQuery, CallAs, Deploy};
use common::create_default_os;
use cosmwasm_std::{coin, Addr, Decimal, Empty};
use cw_staking::CW_STAKING;
use cw_staking::{
    boot::CwStakingApi,
    msg::{CwStakingExecuteMsg, CwStakingQueryMsgFns},
};

use speculoos::*;
use wyndex_bundle::{EUR, EUR_USD_LP, USD, WYNDEX, WYNDEX_OWNER, WYND_TOKEN};

#[test]
fn stake_lp() -> anyhow::Result<()> {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;

    let deployment = Abstract::deploy_on(chain.clone(), "1.0.0".parse()?)?;
    let wyndex = wyndex_bundle::WynDex::deploy_on(chain.clone(), Empty {})?;

    let _root_os = create_default_os(&deployment.os_factory)?;
    let mut staking_api = CwStakingApi::new(CW_STAKING, chain.clone());

    staking_api.deploy("1.0.0".parse()?, Empty {})?;

    let os = create_default_os(&deployment.os_factory)?;
    let proxy_addr = os.proxy.address()?;
    let _manager_addr = os.manager.address()?;

    // transfer some LP tokens to the OS, as if it provided liquidity
    wyndex
        .eur_usd_lp
        .call_as(&Addr::unchecked(WYNDEX_OWNER))
        .transfer(1000, proxy_addr.to_string())?;

    // install exchange on OS
    os.manager.install_module(CW_STAKING, &Empty {})?;
    // load exchange data into type
    staking_api.set_address(&Addr::unchecked(
        os.manager.module_info(CW_STAKING)?.unwrap().address,
    ));

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
