use abstract_adapter::std::{ans_host::QueryMsgFns as _, objects::ABSTRACT_ACCOUNT_ID};
use abstract_dex_adapter::{contract::CONTRACT_VERSION, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_dex_standard::{msg::DexFeesResponse, DexError};
use abstract_interface::{AbstractInterfaceError, AdapterDeployer, DeployStrategy};
use cw20::msg::Cw20ExecuteMsgFns as _;
use cw20_base::msg::QueryMsgFns as _;
mod common;

use abstract_dex_adapter::interface::DexAdapter;
use abstract_interface::{Abstract, AbstractAccount};
use common::create_default_account;
use cosmwasm_std::{coin, Decimal};
use cw_orch::prelude::*;
use speculoos::*;
use wyndex_bundle::{EUR, RAW_TOKEN, USD, WYNDEX as WYNDEX_WITHOUT_CHAIN, WYNDEX_OWNER};

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

    // mint to proxy
    chain.set_balance(&account.proxy.address()?, vec![coin(10_000, EUR)])?;
    // install exchange on OS
    account.install_adapter(&dex_adapter, None)?;

    Ok((chain, wyndex, dex_adapter, account, deployment))
}

#[test]
fn swap_native() -> anyhow::Result<()> {
    let (chain, _, dex_adapter, os, abstr) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    let pools = abstr.ans_host.pool_list(None, None, None)?;
    println!("{:?}", pools);

    // swap 100 EUR to USD
    dex_adapter.ans_swap((EUR, 100), USD, WYNDEX.into(), &os, &abstr.ans_host)?;

    // check balances
    let eur_balance = chain.query_balance(&proxy_addr, EUR)?;
    assert_that!(eur_balance.u128()).is_equal_to(9_900);

    let usd_balance = chain.query_balance(&proxy_addr, USD)?;
    assert_that!(usd_balance.u128()).is_equal_to(98);

    // assert that OS 0 received the swap fee
    let os0_proxy = AbstractAccount::new(&abstr, ABSTRACT_ACCOUNT_ID)
        .proxy
        .address()?;

    let os0_eur_balance = chain.query_balance(&os0_proxy, EUR)?;

    assert_that!(os0_eur_balance.u128()).is_equal_to(1);

    Ok(())
}

#[test]
fn swap_native_without_chain() -> anyhow::Result<()> {
    let (chain, _, dex_adapter, os, abstr) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    // swap 100 EUR to USD
    dex_adapter.ans_swap(
        (EUR, 100),
        USD,
        WYNDEX_WITHOUT_CHAIN.into(),
        &os,
        &abstr.ans_host,
    )?;

    // check balances
    let eur_balance = chain.query_balance(&proxy_addr, EUR)?;
    assert_that!(eur_balance.u128()).is_equal_to(9_900);

    let usd_balance = chain.query_balance(&proxy_addr, USD)?;
    assert_that!(usd_balance.u128()).is_equal_to(98);

    // assert that OS 0 received the swap fee
    let os0_proxy = AbstractAccount::new(&abstr, ABSTRACT_ACCOUNT_ID)
        .proxy
        .address()?;
    let os0_eur_balance = chain.query_balance(&os0_proxy, EUR)?;
    assert_that!(os0_eur_balance.u128()).is_equal_to(1);

    Ok(())
}

#[test]
fn swap_raw() -> anyhow::Result<()> {
    let (chain, wyndex, dex_adapter, os, abstr) = setup_mock()?;
    let proxy_addr = os.proxy.address()?;

    // transfer raw
    let owner = chain.addr_make(WYNDEX_OWNER);
    wyndex
        .raw_token
        .call_as(&owner)
        .transfer(10_000u128, proxy_addr.to_string())?;

    // swap 100 RAW to EUR
    dex_adapter.ans_swap((RAW_TOKEN, 100), EUR, WYNDEX.into(), &os, &abstr.ans_host)?;

    // check balances
    let raw_balance = wyndex.raw_token.balance(proxy_addr.to_string())?;
    assert_that!(raw_balance.balance.u128()).is_equal_to(9_900);

    let eur_balance = chain.query_balance(&proxy_addr, EUR)?;
    assert_that!(eur_balance.u128()).is_equal_to(10098);

    // assert that OS 0 received the swap fee
    let account0_proxy = AbstractAccount::new(&abstr, ABSTRACT_ACCOUNT_ID)
        .proxy
        .address()?;
    let os0_raw_balance = wyndex.raw_token.balance(account0_proxy.to_string())?;
    assert_that!(os0_raw_balance.balance.u128()).is_equal_to(1);

    Ok(())
}

#[test]
fn get_fees() -> anyhow::Result<()> {
    let (_, _, dex_adapter, _, abstr) = setup_mock()?;
    let account0_proxy = AbstractAccount::new(&abstr, ABSTRACT_ACCOUNT_ID)
        .proxy
        .address()?;

    use abstract_dex_adapter::msg::DexQueryMsgFns as _;

    let fees: DexFeesResponse = dex_adapter.fees()?;
    assert_eq!(fees.swap_fee.share(), Decimal::percent(1));
    assert_eq!(fees.recipient, account0_proxy);
    Ok(())
}

#[test]
fn authorized_update_fee() -> anyhow::Result<()> {
    let (_, _, dex_adapter, _, abstr) = setup_mock()?;
    let account0 = AbstractAccount::new(&abstr, ABSTRACT_ACCOUNT_ID);

    let update_fee_msg = abstract_dex_standard::msg::ExecuteMsg::Module(
        abstract_adapter::std::adapter::AdapterRequestMsg {
            proxy_address: Some(account0.proxy.addr_str()?),
            request: abstract_dex_standard::msg::DexExecuteMsg::UpdateFee {
                swap_fee: Some(Decimal::percent(5)),
                recipient_account: None,
            },
        },
    );

    dex_adapter.execute(&update_fee_msg, None)?;

    use abstract_dex_adapter::msg::DexQueryMsgFns as _;

    let fees: DexFeesResponse = dex_adapter.fees()?;
    assert_eq!(fees.swap_fee.share(), Decimal::percent(5));
    Ok(())
}

#[test]
fn unauthorized_update_fee() -> anyhow::Result<()> {
    let (_, _, _, account, _) = setup_mock()?;

    let update_fee_msg = abstract_dex_standard::msg::ExecuteMsg::Module(
        abstract_adapter::std::adapter::AdapterRequestMsg {
            proxy_address: None,
            request: abstract_dex_standard::msg::DexExecuteMsg::UpdateFee {
                swap_fee: Some(Decimal::percent(5)),
                recipient_account: None,
            },
        },
    );

    let err = account
        .manager
        .execute_on_module(DEX_ADAPTER_ID, update_fee_msg)
        .unwrap_err();
    let AbstractInterfaceError::Orch(orch_error) = err else {
        panic!("unexpected error type");
    };
    let dex_err: DexError = orch_error.downcast().unwrap();
    assert_eq!(dex_err, DexError::Unauthorized {});
    Ok(())
}
