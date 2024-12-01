use abstract_adapter::std::{ans_host::QueryMsgFns as _, objects::ABSTRACT_ACCOUNT_ID};
use abstract_client::builder::cw20_builder::{ExecuteMsgInterfaceFns, QueryMsgInterfaceFns};
use abstract_dex_adapter::{contract::CONTRACT_VERSION, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_dex_standard::{msg::DexFeesResponse, DexError};
use abstract_interface::{AbstractInterfaceError, AccountI, AdapterDeployer, DeployStrategy};

use abstract_dex_adapter::interface::DexAdapter;
use abstract_integration_tests::create_default_account;
use abstract_interface::Abstract;
use cosmwasm_std::{coin, Decimal};
use cw_orch::prelude::*;
use cw_plus_orch::cw20_base::interfaces::QueryMsgInterfaceFns;
use mockdex_bundle::{EUR, RAW_TOKEN, USD, WYNDEX as WYNDEX_WITHOUT_CHAIN, WYNDEX_OWNER};

const WYNDEX: &str = "cosmos-testnet>wyndex";

#[allow(clippy::type_complexity)]
fn setup_mock() -> anyhow::Result<(
    MockBech32,
    mockdex_bundle::WynDex,
    DexAdapter<MockBech32>,
    AccountI<MockBech32>,
    Abstract<MockBech32>,
)> {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let wyndex = mockdex_bundle::WynDex::deploy_on(chain.clone(), Empty {})?;

    let _root_os = create_default_account(&sender, &deployment)?;
    let dex_adapter = DexAdapter::new(DEX_ADAPTER_ID, chain.clone());

    dex_adapter.deploy(
        CONTRACT_VERSION.parse()?,
        DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: ABSTRACT_ACCOUNT_ID.seq(),
        },
        DeployStrategy::Try,
    )?;

    let account = create_default_account(&sender, &deployment)?;

    // mint to account
    chain.set_balance(&account.address()?, vec![coin(10_000, EUR)])?;
    // install exchange on OS
    account.install_adapter(&dex_adapter, &[])?;

    Ok((chain, wyndex, dex_adapter, account, deployment))
}

#[test]
fn swap_native() -> anyhow::Result<()> {
    let (chain, _, dex_adapter, account, abstr) = setup_mock()?;
    let account_addr = account.address()?;

    let pools = abstr.ans_host.pool_list(None, None, None)?;
    println!("{:?}", pools);

    // swap 100 EUR to USD
    dex_adapter.ans_swap((EUR, 100), USD, WYNDEX.into(), &account, &abstr.ans_host)?;

    // check balances
    let eur_balance = chain.query_balance(&account_addr, EUR)?;
    assert_eq!(eur_balance.u128(), 9_900);

    let usd_balance = chain.query_balance(&account_addr, USD)?;
    assert_eq!(usd_balance.u128(), 98);

    // assert that OS 0 received the swap fee
    let os0_account = AccountI::load_from(&abstr, ABSTRACT_ACCOUNT_ID)?.address()?;

    let os0_eur_balance = chain.query_balance(&os0_account, EUR)?;

    assert_eq!(os0_eur_balance.u128(), 1);

    Ok(())
}

#[test]
fn swap_native_without_chain() -> anyhow::Result<()> {
    let (chain, _, dex_adapter, account, abstr) = setup_mock()?;
    let account_addr = account.address()?;

    // swap 100 EUR to USD
    dex_adapter.ans_swap(
        (EUR, 100),
        USD,
        WYNDEX_WITHOUT_CHAIN.into(),
        &account,
        &abstr.ans_host,
    )?;

    // check balances
    let eur_balance = chain.query_balance(&account_addr, EUR)?;
    assert_eq!(eur_balance.u128(), 9_900);

    let usd_balance = chain.query_balance(&account_addr, USD)?;
    assert_eq!(usd_balance.u128(), 98);

    // assert that OS 0 received the swap fee
    let os0_account = AccountI::load_from(&abstr, ABSTRACT_ACCOUNT_ID)?.address()?;
    let os0_eur_balance = chain.query_balance(&os0_account, EUR)?;
    assert_eq!(os0_eur_balance.u128(), 1);

    Ok(())
}

#[test]
fn swap_raw() -> anyhow::Result<()> {
    let (chain, wyndex, dex_adapter, account, abstr) = setup_mock()?;
    let account_addr = account.address()?;

    // transfer raw
    let owner = chain.addr_make(WYNDEX_OWNER);
    wyndex
        .raw_token
        .call_as(&owner)
        .transfer(10_000u128, account_addr.to_string())?;

    // swap 100 RAW to EUR
    dex_adapter.ans_swap(
        (RAW_TOKEN, 100),
        EUR,
        WYNDEX.into(),
        &account,
        &abstr.ans_host,
    )?;

    // check balances
    let raw_balance = wyndex.raw_token.balance(account_addr.to_string())?;
    assert_eq!(raw_balance.balance.u128(), 9_900);

    let eur_balance = chain.query_balance(&account_addr, EUR)?;
    assert_eq!(eur_balance.u128(), 10098);

    // assert that OS 0 received the swap fee
    let account0_account = AccountI::load_from(&abstr, ABSTRACT_ACCOUNT_ID)?.address()?;
    let os0_raw_balance = wyndex.raw_token.balance(account0_account.to_string())?;
    assert_eq!(os0_raw_balance.balance.u128(), 1);

    Ok(())
}

#[test]
fn get_fees() -> anyhow::Result<()> {
    let (_, _, dex_adapter, _, abstr) = setup_mock()?;
    let account0_account = AccountI::load_from(&abstr, ABSTRACT_ACCOUNT_ID)?.address()?;

    use abstract_dex_adapter::msg::DexQueryMsgFns as _;

    let fees: DexFeesResponse = dex_adapter.fees()?;
    assert_eq!(fees.swap_fee.share(), Decimal::percent(1));
    assert_eq!(fees.recipient, account0_account);
    Ok(())
}

#[test]
fn authorized_update_fee() -> anyhow::Result<()> {
    let (_, _, dex_adapter, _, abstr) = setup_mock()?;
    let account0 = AccountI::load_from(&abstr, ABSTRACT_ACCOUNT_ID)?;

    let update_fee_msg = abstract_dex_standard::msg::ExecuteMsg::Module(
        abstract_adapter::std::adapter::AdapterRequestMsg {
            account_address: Some(account0.addr_str()?),
            request: abstract_dex_standard::msg::DexExecuteMsg::UpdateFee {
                swap_fee: Some(Decimal::percent(5)),
                recipient_account: None,
            },
        },
    );

    dex_adapter.execute(&update_fee_msg, &[])?;

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
            account_address: None,
            request: abstract_dex_standard::msg::DexExecuteMsg::UpdateFee {
                swap_fee: Some(Decimal::percent(5)),
                recipient_account: None,
            },
        },
    );

    let err = account
        .execute_on_module(DEX_ADAPTER_ID, update_fee_msg, vec![])
        .unwrap_err();
    let AbstractInterfaceError::Orch(orch_error) = err else {
        panic!("unexpected error type");
    };
    let dex_err: DexError = orch_error.downcast().unwrap();
    assert_eq!(dex_err, DexError::Unauthorized {});
    Ok(())
}
