use abstract_dex_adapter::contract::CONTRACT_VERSION;
use abstract_dex_adapter::msg::DexInstantiateMsg;
use abstract_dex_adapter::EXCHANGE;
use abstract_interface::AdapterDeployer;
use cw20::msg::Cw20ExecuteMsgFns;
use cw20_base::msg::QueryMsgFns;
use cw_orch::deploy::Deploy;
mod common;

use abstract_dex_adapter::interface::DexAdapter;
use abstract_interface::Abstract;
use abstract_interface::AbstractAccount;
use common::create_default_account;
use cosmwasm_std::{coin, Addr, Decimal, Empty};

use cw_orch::prelude::*;
use speculoos::*;
use wyndex_bundle::{EUR, RAW_TOKEN, USD, WYNDEX as WYNDEX_WITHOUT_CHAIN, WYNDEX_OWNER};

const WYNDEX: &str = "cosmos-testnet>wynd";

#[allow(clippy::type_complexity)]
fn setup_mock() -> anyhow::Result<(
    Mock,
    wyndex_bundle::WynDex,
    DexAdapter<Mock>,
    AbstractAccount<Mock>,
    Abstract<Mock>,
)> {
    let sender = Addr::unchecked(common::ROOT_USER);
    let chain = Mock::new(&sender);

    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;
    let wyndex = wyndex_bundle::WynDex::deploy_on(chain.clone(), Empty {})?;

    let _root_os = create_default_account(&deployment.account_factory)?;
    let dex_adapter = DexAdapter::new(EXCHANGE, chain.clone());

    dex_adapter.deploy(
        CONTRACT_VERSION.parse()?,
        DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: 0,
        },
    )?;

    let account = create_default_account(&deployment.account_factory)?;

    // mint to proxy
    chain.set_balance(&account.proxy.address()?, vec![coin(10_000, EUR)])?;
    // install exchange on OS
    account.manager.install_module(EXCHANGE, &Empty {}, None)?;
    // load exchange data into type
    dex_adapter.set_address(&Addr::unchecked(
        account.manager.module_info(EXCHANGE)?.unwrap().address,
    ));

    Ok((chain, wyndex, dex_adapter, account, deployment))
}

#[test]
fn swap_native() -> anyhow::Result<()> {
    let (chain, _, dex_adapter, os, abstr) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    // swap 100 EUR to USD
    dex_adapter.swap((EUR, 100), USD, WYNDEX.into())?;

    // check balances
    let eur_balance = chain.query_balance(&proxy_addr, EUR)?;
    assert_that!(eur_balance.u128()).is_equal_to(9_900);

    let usd_balance = chain.query_balance(&proxy_addr, USD)?;
    assert_that!(usd_balance.u128()).is_equal_to(98);

    // assert that OS 0 received the swap fee
    let os0_proxy = AbstractAccount::new(&abstr, Some(0)).proxy.address()?;
    let os0_eur_balance = chain.query_balance(&os0_proxy, EUR)?;
    assert_that!(os0_eur_balance.u128()).is_equal_to(1);

    Ok(())
}

#[test]
fn swap_native_without_chain() -> anyhow::Result<()> {
    let (chain, _, dex_adapter, os, abstr) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    // swap 100 EUR to USD
    dex_adapter.swap((EUR, 100), USD, WYNDEX_WITHOUT_CHAIN.into())?;

    // check balances
    let eur_balance = chain.query_balance(&proxy_addr, EUR)?;
    assert_that!(eur_balance.u128()).is_equal_to(9_900);

    let usd_balance = chain.query_balance(&proxy_addr, USD)?;
    assert_that!(usd_balance.u128()).is_equal_to(98);

    // assert that OS 0 received the swap fee
    let os0_proxy = AbstractAccount::new(&abstr, Some(0)).proxy.address()?;
    let os0_eur_balance = chain.query_balance(&os0_proxy, EUR)?;
    assert_that!(os0_eur_balance.u128()).is_equal_to(1);

    Ok(())
}

#[test]
fn swap_raw() -> anyhow::Result<()> {
    let (chain, wyndex, dex_adapter, os, abstr) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    // trnasfer raw
    let owner = Addr::unchecked(WYNDEX_OWNER);
    wyndex
        .raw_token
        .call_as(&owner)
        .transfer(10_000u128.into(), proxy_addr.to_string())?;

    // swap 100 RAW to EUR
    dex_adapter.swap((RAW_TOKEN, 100), EUR, WYNDEX.into())?;

    // check balances
    let raw_balance = wyndex.raw_token.balance(proxy_addr.to_string())?;
    assert_that!(raw_balance.balance.u128()).is_equal_to(9_900);

    let eur_balance = chain.query_balance(&proxy_addr, EUR)?;
    assert_that!(eur_balance.u128()).is_equal_to(10098);

    // assert that OS 0 received the swap fee
    let account0_proxy = AbstractAccount::new(&abstr, Some(0)).proxy.address()?;
    let os0_raw_balance = wyndex.raw_token.balance(account0_proxy.to_string())?;
    assert_that!(os0_raw_balance.balance.u128()).is_equal_to(1);

    Ok(())
}
