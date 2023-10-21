use abstract_core::{
    ans_host::ExecuteMsgFns,
    objects::{gov_type::GovernanceDetails, AccountId, AssetEntry},
};
use abstract_dex_adapter::{contract::CONTRACT_VERSION, msg::DexInstantiateMsg, DEX_ADAPTER_ID};
use abstract_interface::{
    Abstract, AbstractAccount, AdapterDeployer, AppDeployer, DeployStrategy, ManagerQueryFns,
    VCExecFns,
};
use cw20::{Cw20Coin, Cw20ExecuteMsgFns};
use cw20_base::{
    contract::Cw20Base as AbstractCw20Base,
    msg::{InstantiateMsg as Cw20InstantiateMsg, QueryMsgFns},
};
use payment_app::{
    contract::{APP_ID, APP_VERSION},
    msg::{AppInstantiateMsg, ConfigResponse, TipCountResponse, TipperResponse, TippersResponse},
    *,
};
use wyndex_bundle::WynDex;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{coin, coins, to_binary, Addr, Decimal, Uint128};

// consts for testing
const ADMIN: &str = "admin";

/// Set up the test environment with the contract installed
fn setup(
    mock: Mock,
    desired_asset: Option<AssetEntry>,
    should_load_abstract: bool,
) -> anyhow::Result<(
    AbstractAccount<Mock>,
    Abstract<Mock>,
    PaymentAppInterface<Mock>,
)> {
    // Construct the counter interface
    let app = PaymentAppInterface::new(APP_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = if should_load_abstract {
        Abstract::load_from(mock.clone())?
    } else {
        Abstract::deploy_on(mock.clone(), mock.sender().to_string())?
    };

    let dex_adapter = abstract_dex_adapter::interface::DexAdapter::new(
        abstract_dex_adapter::DEX_ADAPTER_ID,
        mock,
    );
    dex_adapter.deploy(
        CONTRACT_VERSION.parse().unwrap(),
        DexInstantiateMsg {
            recipient_account: 0,
            swap_fee: Decimal::percent(1),
        },
        DeployStrategy::Try,
    )?;

    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: ADMIN.to_string(),
            })?;

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespace(AccountId::local(1), "my-namespace".to_string())?;

    app.deploy(APP_VERSION.parse()?, DeployStrategy::Try)?;

    // install exchange module as it's a dependency
    account.install_module(DEX_ADAPTER_ID, &Empty {}, None)?;

    account.install_app(
        app.clone(),
        &AppInstantiateMsg {
            desired_asset,
            exchanges: vec!["wyndex".to_string()],
        },
        None,
    )?;

    let modules = account.manager.module_infos(None, None)?;
    app.set_address(&modules.module_infos[1].address);

    account.manager.update_adapter_authorized_addresses(
        abstract_dex_adapter::DEX_ADAPTER_ID,
        vec![app.address()?.to_string()],
        vec![],
    )?;

    Ok((account, abstr_deployment, app))
}

fn wyndex_deployment(chain: &Mock) -> WynDex {
    WynDex::store_on(chain.clone()).unwrap()
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Set up the environment and contract
    let (_account, _abstr, app) = setup(mock, None, false)?;

    let config = app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            desired_asset: None,
            exchanges: vec!["wyndex".to_string()]
        }
    );
    Ok(())
}

#[test]
fn test_update_config() -> anyhow::Result<()> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Set up the environment and contract
    let (account, _abstr, app) = setup(mock, None, false)?;

    let dex_name = String::from("osmosis");

    app.call_as(&account.manager.address()?)
        .update_config(Some(vec![dex_name.clone()]))?;

    let config = app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            desired_asset: None,
            exchanges: vec![dex_name]
        }
    );
    Ok(())
}

#[test]
fn test_simple_tip() -> anyhow::Result<()> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    let (account, abstr_deployment, app) = setup(mock, None, false)?;
    let mock: Mock = abstr_deployment.ans_host.get_chain().clone();
    let WynDex {
        eur_token,
        usd_token: _,
        eur_usd_lp: _,
        ..
    } = wyndex_deployment(&mock);
    let tipper = Addr::unchecked("tipper");
    let tip_amount = 100;
    let tip_currency = eur_token.to_string();
    let tip_coins = coins(tip_amount, tip_currency.clone());
    mock.set_balance(&tipper, tip_coins.clone())?;

    app.call_as(&tipper).tip(&tip_coins)?;

    let balance = mock.query_balance(&account.proxy.address()?, &tip_currency)?;
    assert_eq!(balance, Uint128::from(tip_amount));

    // Query tip count
    let tip_count_response: TipCountResponse = app.tip_count()?;
    assert_eq!(1, tip_count_response.count);

    // Query tipper
    let tipper_response: TipperResponse = app.tipper(tipper.to_string())?;
    let expected_tipper = TipperResponse {
        address: tipper,
        count: 1,
        total_amount: Uint128::zero(),
    };
    assert_eq!(expected_tipper, tipper_response);

    // List tippers
    let tippers_response: TippersResponse = app.list_tippers(None, None)?;
    let tippers = tippers_response.tippers;
    assert_eq!(vec![tipper_response], tippers);

    Ok(())
}

#[test]
fn test_tip_swap() -> anyhow::Result<()> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Deploy Abstract to the mock
    Abstract::deploy_on(mock.clone(), sender.to_string())?;

    let WynDex {
        eur_token,
        usd_token,
        eur_usd_lp: _,
        ..
    } = wyndex_deployment(&mock);

    let tipper = Addr::unchecked("tipper");
    let tip_amount = 100;
    let tip_currency = eur_token.to_string();
    let target_currency = usd_token.to_string();
    let tip_coins = coins(tip_amount, tip_currency.clone());
    mock.set_balance(&tipper, tip_coins.clone())?;

    let (account, _abstr_deployment, app) =
        setup(mock.clone(), Some(AssetEntry::new(&target_currency)), true)?;

    app.call_as(&tipper).tip(&tip_coins)?;

    let balance = mock.query_balance(&account.proxy.address()?, &target_currency)?;
    assert!(!balance.is_zero());
    let balance = mock.query_balance(&account.proxy.address()?, &tip_currency)?;
    assert!(balance.is_zero());

    // Query tip count
    let tip_count_response: TipCountResponse = app.tip_count()?;
    assert_eq!(1, tip_count_response.count);

    // Query tipper
    let tipper_response: TipperResponse = app.tipper(tipper.to_string())?;
    assert_eq!(1, tipper_response.count);
    assert!(!tipper_response.total_amount.is_zero());

    // List tippers
    let tippers_response: TippersResponse = app.list_tippers(None, None)?;
    let tippers = tippers_response.tippers;
    assert_eq!(1, tippers.len());
    assert_eq!(tipper_response, tippers[0]);

    Ok(())
}

#[test]
fn test_tip_swap_and_not_swap() -> anyhow::Result<()> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Deploy Abstract to the mock
    Abstract::deploy_on(mock.clone(), sender.to_string())?;

    let WynDex {
        eur_token,
        usd_token,
        eur_usd_lp: _,
        ..
    } = wyndex_deployment(&mock);

    let tipper = Addr::unchecked("tipper");
    let tip_amount = 100;
    let tip_amount1 = 100;
    let tip_currency = eur_token.to_string();
    let tip_currency1 = "gm";
    let target_currency = usd_token.to_string();
    let tip_coins = vec![
        coin(tip_amount, tip_currency.clone()),
        coin(tip_amount1, tip_currency1),
        coin(tip_amount1, target_currency.clone()),
    ];
    mock.set_balance(&tipper, tip_coins.clone())?;

    let (account, abstr_deployment, app) =
        setup(mock.clone(), Some(AssetEntry::new(&target_currency)), true)?;

    // We add the currency gm to the abstract deployment
    abstr_deployment
        .ans_host
        .update_asset_addresses(
            vec![(
                tip_currency1.to_string(),
                cw_asset::AssetInfoBase::native(tip_currency1.to_string()),
            )],
            vec![],
        )
        .unwrap();

    app.call_as(&tipper).tip(&tip_coins)?;

    let balance = mock.query_balance(&account.proxy.address()?, &tip_currency)?;
    assert!(balance.is_zero());

    let balance = mock.query_balance(&account.proxy.address()?, &target_currency)?;
    assert!(!balance.is_zero());

    let balance = mock.query_balance(&account.proxy.address()?, tip_currency1)?;
    assert_eq!(balance, Uint128::from(tip_amount1));

    Ok(())
}

#[test]
fn test_cw20_tip() -> anyhow::Result<()> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Deploy Abstract to the mock
    Abstract::deploy_on(mock.clone(), sender.to_string())?;

    let WynDex { usd_token, .. } = wyndex_deployment(&mock);

    let tipper = Addr::unchecked("tipper");
    let tip_amount = 100u128;
    let starting_balance = 1000u128;

    let cw20_token_name = "cw20_token";
    let cw20_token_ticker = "token";
    let cw20_token = AbstractCw20Base::new(cw20_token_name, mock.clone());
    cw20_token.upload()?;
    cw20_token.instantiate(
        &Cw20InstantiateMsg {
            name: cw20_token_name.to_owned(),
            symbol: cw20_token_ticker.to_owned(),
            decimals: 18,
            initial_balances: vec![Cw20Coin {
                address: tipper.to_string(),
                amount: Uint128::from(starting_balance),
            }],
            mint: None,
            marketing: None,
        },
        None,
        None,
    )?;

    let target_currency = usd_token.to_string();

    let (account, abstr_deployment, app) =
        setup(mock.clone(), Some(AssetEntry::new(&target_currency)), true)?;

    // We add the currency cw20 to the abstract deployment
    abstr_deployment
        .ans_host
        .update_asset_addresses(
            vec![(
                cw20_token.address()?.to_string(),
                cw_asset::AssetInfoBase::cw20(cw20_token.address()?.to_string()),
            )],
            vec![],
        )
        .unwrap();

    cw20_token.call_as(&tipper).send(
        Uint128::from(tip_amount),
        app.address()?.to_string(),
        to_binary("")?,
    )?;

    let tipper_balance = cw20_token.balance(tipper.to_string())?.balance;
    assert_eq!(starting_balance - tip_amount, tipper_balance.u128());

    let proxy_balance = cw20_token
        .balance(account.proxy.address()?.to_string())?
        .balance;
    assert_eq!(tip_amount, proxy_balance.u128());

    // Query tip count
    let tip_count_response: TipCountResponse = app.tip_count()?;
    assert_eq!(1, tip_count_response.count);

    // Query tipper
    let tipper_response: TipperResponse = app.tipper(tipper.to_string())?;
    let expected_tipper = TipperResponse {
        address: tipper,
        count: 1,
        total_amount: Uint128::zero(),
    };
    assert_eq!(expected_tipper, tipper_response);

    Ok(())
}

#[test]
fn test_multiple_tippers() -> anyhow::Result<()> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    let (account, abstr_deployment, app) = setup(mock, None, false)?;
    let mock: Mock = abstr_deployment.ans_host.get_chain().clone();
    let WynDex {
        eur_token,
        usd_token: _,
        eur_usd_lp: _,
        ..
    } = wyndex_deployment(&mock);

    // First tipper
    let tipper1 = Addr::unchecked("tipper1");
    let tip_amount1 = 100;
    let tip_currency = eur_token.to_string();
    let tip_coins = coins(tip_amount1, tip_currency.clone());
    mock.set_balance(&tipper1, tip_coins.clone())?;

    app.call_as(&tipper1).tip(&tip_coins)?;

    // Second tipper
    let tipper2 = Addr::unchecked("tipper2");
    let tip_amount2 = 200;
    let tip_currency = eur_token.to_string();
    let tip_coins = coins(tip_amount2, tip_currency.clone());
    mock.set_balance(&tipper2, tip_coins.clone())?;

    app.call_as(&tipper2).tip(&tip_coins)?;

    let balance = mock.query_balance(&account.proxy.address()?, &tip_currency)?;
    assert_eq!(balance, Uint128::from(tip_amount1 + tip_amount2));

    // Query tip count
    let tip_count_response: TipCountResponse = app.tip_count()?;
    assert_eq!(2, tip_count_response.count);

    // Query first tipper
    let tipper_response1: TipperResponse = app.tipper(tipper1.to_string())?;
    let expected_tipper = TipperResponse {
        address: tipper1,
        count: 1,
        total_amount: Uint128::zero(),
    };
    assert_eq!(expected_tipper, tipper_response1);

    // Query second tipper
    let tipper_response2: TipperResponse = app.tipper(tipper2.to_string())?;
    let expected_tipper = TipperResponse {
        address: tipper2,
        count: 1,
        total_amount: Uint128::zero(),
    };
    assert_eq!(expected_tipper, tipper_response2);

    // List tippers
    let tippers_response: TippersResponse = app.list_tippers(None, None)?;
    let tippers = tippers_response.tippers;
    assert_eq!(vec![tipper_response1, tipper_response2], tippers);

    Ok(())
}
