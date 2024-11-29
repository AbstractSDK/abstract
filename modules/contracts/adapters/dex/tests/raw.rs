use abstract_adapter::std::{
    adapter::AdapterRequestMsg,
    ans_host::QueryMsgFns as _,
    objects::{PoolAddress, ABSTRACT_ACCOUNT_ID},
};
use abstract_client::builder::cw20_builder::{ExecuteMsgInterfaceFns, QueryMsgInterfaceFns};
use abstract_dex_adapter::interface::DexAdapter;
use abstract_dex_adapter::{contract::CONTRACT_VERSION, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_dex_standard::action::DexAction;
use abstract_dex_standard::msg::DexExecuteMsg;
use abstract_integration_tests::create_default_account;
use abstract_interface::Abstract;
use abstract_interface::{AccountI, AdapterDeployer, DeployStrategy};
use cosmwasm_std::{coin, Decimal};
use cw_asset::{AssetBase, AssetInfoBase};
use cw_orch::prelude::*;
use mockdex_bundle::{EUR, USD, WYNDEX as WYNDEX_WITHOUT_CHAIN, WYNDEX_OWNER};

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
fn raw_swap_native() -> anyhow::Result<()> {
    let (chain, wyndex, dex_adapter, account, abstr) = setup_mock()?;
    let account_addr = account.address()?;

    let pools = abstr.ans_host.pool_list(None, None, None)?;
    println!("{:?}", pools);

    // swap 100 EUR to USD
    dex_adapter.raw_swap_native(
        (EUR, 100),
        USD,
        WYNDEX.into(),
        &account,
        PoolAddress::contract(wyndex.eur_usd_pair).into(),
    )?;

    // check balances
    let eur_balance = chain.query_balance(&account_addr, EUR)?;
    assert_eq!(eur_balance.u128(), 9_900);

    let usd_balance = chain.query_balance(&account_addr, USD)?;
    assert_eq!(usd_balance.u128(), 98);

    // assert that account 0 received the swap fee
    let os0_account = AccountI::load_from(&abstr, ABSTRACT_ACCOUNT_ID)?.address()?;

    let os0_eur_balance = chain.query_balance(&os0_account, EUR)?;

    assert_eq!(os0_eur_balance.u128(), 1);

    Ok(())
}

#[test]
fn raw_swap_native_without_chain() -> anyhow::Result<()> {
    let (chain, wyndex, dex_adapter, account, abstr) = setup_mock()?;
    let account_addr = account.address()?;

    // swap 100 EUR to USD
    dex_adapter.raw_swap_native(
        (EUR, 100),
        USD,
        WYNDEX_WITHOUT_CHAIN.into(),
        &account,
        PoolAddress::contract(wyndex.eur_usd_pair).into(),
    )?;

    // check balances
    let balances = chain.query_all_balances(&account_addr)?;
    println!("{:?}", balances);
    let eur_balance = chain.query_balance(&account_addr, EUR)?;
    assert_eq!(eur_balance.u128(), 9_900);

    let usd_balance = chain.query_balance(&account_addr, USD)?;
    assert_eq!(usd_balance.u128(), 98);

    // assert that Account 0 received the swap fee
    let account = AccountI::load_from(&abstr, ABSTRACT_ACCOUNT_ID)?.address()?;
    let eur_balance = chain.query_balance(&account, EUR)?;
    assert_eq!(eur_balance.u128(), 1);

    Ok(())
}

#[test]
fn raw_swap_raw() -> anyhow::Result<()> {
    let (chain, wyndex, _, account, abstr) = setup_mock()?;
    let account_addr = account.address()?;

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
    account.execute_on_module(DEX_ADAPTER_ID, swap_msg, vec![])?;

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
