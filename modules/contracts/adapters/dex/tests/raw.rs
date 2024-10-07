#![cfg(feature = "TODO: replace wyndex_bundle")]

use abstract_adapter::std::{
    adapter::AdapterRequestMsg,
    ans_host::QueryMsgFns as _,
    objects::{PoolAddress, ABSTRACT_ACCOUNT_ID},
};
use abstract_dex_adapter::{contract::CONTRACT_VERSION, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_dex_standard::msg::DexExecuteMsg;
use abstract_interface::{AdapterDeployer, DeployStrategy};
use cw20::msg::Cw20ExecuteMsgFns as _;
use cw20_base::msg::QueryMsgFns as _;
use cw_asset::{AssetBase, AssetInfoBase};
mod common;
use abstract_dex_adapter::interface::DexAdapter;
use abstract_dex_standard::action::DexAction;
use abstract_interface::{Abstract, AbstractAccount};
use common::create_default_account;
use cosmwasm_std::{coin, Decimal};
use cw_orch::prelude::*;
use speculoos::*;
use wyndex_bundle::{EUR, USD, WYNDEX as WYNDEX_WITHOUT_CHAIN, WYNDEX_OWNER};

const WYNDEX: &str = "cosmos-testnet>wyndex";

#[allow(clippy::type_complexity)]
fn setup_mock() -> anyhow::Result<(
    MockBech32,
    wyndex_bundle::WynDex,
    DexAdapter<MockBech32>,
    AbstractAccount<MockBech32>,
    Abstract<MockBech32>,
)> {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let wyndex = wyndex_bundle::WynDex::deploy_on(chain.clone(), Empty {})?;

    let _root_os = create_default_account(&deployment.account_factory)?;
    let dex_adapter = DexAdapter::new(DEX_ADAPTER_ID, chain.clone());

    dex_adapter.deploy(
        CONTRACT_VERSION.parse()?,
        DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: ABSTRACT_ACCOUNT_ID.seq(),
        },
        DeployStrategy::Try,
    )?;

    let account = create_default_account(&deployment.account_factory)?;

    // mint to account
    chain.set_balance(&account.address()?, vec![coin(10_000, EUR)])?;
    // install exchange on OS
    account.install_adapter(&dex_adapter, None)?;

    Ok((chain, wyndex, dex_adapter, account, deployment))
}

#[test]
fn raw_swap_native() -> anyhow::Result<()> {
    let (chain, wyndex, dex_adapter, os, abstr) = setup_mock()?;
    let account_addr = os.account.address()?;

    let pools = abstr.ans_host.pool_list(None, None, None)?;
    println!("{:?}", pools);

    // swap 100 EUR to USD
    dex_adapter.raw_swap_native(
        (EUR, 100),
        USD,
        WYNDEX.into(),
        &os,
        PoolAddress::contract(wyndex.eur_usd_pair).into(),
    )?;

    // check balances
    let eur_balance = chain.query_balance(&account_addr, EUR)?;
    assert_that!(eur_balance.u128()).is_equal_to(9_900);

    let usd_balance = chain.query_balance(&account_addr, USD)?;
    assert_that!(usd_balance.u128()).is_equal_to(98);

    // assert that OS 0 received the swap fee
    let os0_account = AbstractAccount::new(&abstr, ABSTRACT_ACCOUNT_ID)
        .account
        .address()?;

    let os0_eur_balance = chain.query_balance(&os0_account, EUR)?;

    assert_that!(os0_eur_balance.u128()).is_equal_to(1);

    Ok(())
}

#[test]
fn raw_swap_native_without_chain() -> anyhow::Result<()> {
    let (chain, wyndex, dex_adapter, os, abstr) = setup_mock()?;
    let account_addr = os.account.address()?;

    // swap 100 EUR to USD
    dex_adapter.raw_swap_native(
        (EUR, 100),
        USD,
        WYNDEX_WITHOUT_CHAIN.into(),
        &os,
        PoolAddress::contract(wyndex.eur_usd_pair).into(),
    )?;

    // check balances
    let balances = chain.query_all_balances(&account_addr)?;
    println!("{:?}", balances);
    let eur_balance = chain.query_balance(&account_addr, EUR)?;
    assert_that!(eur_balance.u128()).is_equal_to(9_900);

    let usd_balance = chain.query_balance(&account_addr, USD)?;
    assert_that!(usd_balance.u128()).is_equal_to(98);

    // assert that OS 0 received the swap fee
    let os0_account = AbstractAccount::new(&abstr, ABSTRACT_ACCOUNT_ID)
        .account
        .address()?;
    let os0_eur_balance = chain.query_balance(&os0_account, EUR)?;
    assert_that!(os0_eur_balance.u128()).is_equal_to(1);

    Ok(())
}

#[test]
fn raw_swap_raw() -> anyhow::Result<()> {
    let (chain, wyndex, _, os, abstr) = setup_mock()?;
    let account_addr = os.account.address()?;

    // transfer raw
    let owner = chain.addr_make(WYNDEX_OWNER);
    wyndex
        .raw_token
        .call_as(&owner)
        .transfer(10_000u128, account_addr.to_string())?;

    // swap 100 RAW to EUR
    let swap_msg = abstract_dex_adapter::msg::ExecuteMsg::Module(AdapterRequestMsg {
        account_address: None,
        request: DexExecuteMsg::Action {
            dex: WYNDEX.to_owned(),
            action: DexAction::Swap {
                offer_asset: AssetBase::cw20(wyndex.raw_token.address()?.to_string(), 100u128),
                ask_asset: AssetInfoBase::native(EUR),
                pool: PoolAddress::contract(wyndex.raw_eur_pair).into(),
                max_spread: Some(Decimal::percent(30)),
                belief_price: None,
            },
        },
    });
    os.account.execute_on_module(DEX_ADAPTER_ID, swap_msg)?;

    // check balances
    let raw_balance = wyndex.raw_token.balance(account_addr.to_string())?;
    assert_that!(raw_balance.balance.u128()).is_equal_to(9_900);

    let eur_balance = chain.query_balance(&account_addr, EUR)?;
    assert_that!(eur_balance.u128()).is_equal_to(10098);

    // assert that OS 0 received the swap fee
    let account0_account = AbstractAccount::new(&abstr, ABSTRACT_ACCOUNT_ID)
        .account
        .address()?;
    let os0_raw_balance = wyndex.raw_token.balance(account0_account.to_string())?;
    assert_that!(os0_raw_balance.balance.u128()).is_equal_to(1);

    Ok(())
}
