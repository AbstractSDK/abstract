#![cfg(feature = "TODO: replace wyndex_bundle")]

use abstract_app::sdk::cw_helpers::Clearable;
use abstract_app::std::{
    ans_host::ExecuteMsgFns,
    objects::{gov_type::GovernanceDetails, AccountId, AnsAsset, AssetEntry},
};
use abstract_dex_adapter::{contract::CONTRACT_VERSION, msg::DexInstantiateMsg};
use abstract_interface::{
    Abstract, AbstractAccount, AdapterDeployer, AppDeployer, DeployStrategy, RegistryExecFns,
};
use cosmwasm_std::{coin, coins, to_json_binary, Decimal, Uint128};
use cw20::{msg::Cw20ExecuteMsgFns, Cw20Coin};
use cw20_base::msg::{InstantiateMsg as Cw20InstantiateMsg, QueryMsgFns};
use cw_orch::environment::Environment;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};
use cw_plus_orch::cw20_base::Cw20Base as AbstractCw20Base;
use payment_app::{
    contract::{APP_ID, APP_VERSION},
    msg::{
        AppInstantiateMsg, ConfigResponse, TipCountResponse, TipperCountResponse, TipperResponse,
        TippersCountResponse,
    },
    *,
};
use wyndex_bundle::WynDex;

type PaymentTestSetup = (
    AbstractAccount<MockBech32>,
    Abstract<MockBech32>,
    PaymentAppInterface<MockBech32>,
    WynDex,
);
/// Set up the test environment with the contract installed
fn setup(mock: MockBech32, desired_asset: Option<AssetEntry>) -> anyhow::Result<PaymentTestSetup> {
    let app = PaymentAppInterface::new(APP_ID, mock.clone());

    let abstr_deployment = Abstract::deploy_on(mock.clone(), mock.sender_addr().to_string())?;

    let dex_adapter = abstract_dex_adapter::interface::DexAdapter::new(
        abstract_dex_adapter::DEX_ADAPTER_ID,
        mock.clone(),
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
                monarch: mock.sender_addr().to_string(),
            })?;

    // claim the namespace so app can be deployed
    abstr_deployment
        .registry
        .claim_namespace(AccountId::local(1), "my-namespace".to_string())?;

    app.deploy(APP_VERSION.parse()?, DeployStrategy::Try)?;

    // install exchange module as it's a dependency
    account.install_adapter(&dex_adapter, None)?;

    let wyndex = WynDex::store_on(mock)?;

    account.install_app(
        &app,
        &AppInstantiateMsg {
            desired_asset,
            denom_asset: "Dollah".to_owned(),
            exchanges: vec![wyndex_bundle::WYNDEX.to_owned()],
        },
        None,
    )?;

    account.account.update_adapter_authorized_addresses(
        abstract_dex_adapter::DEX_ADAPTER_ID,
        vec![app.address()?.to_string()],
        vec![],
    )?;

    Ok((account, abstr_deployment, app, wyndex))
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Create the mock
    let mock = MockBech32::new("sender");

    // Set up the environment and contract
    let (_account, _abstr, app, _wyndex) = setup(mock, None)?;

    let config = app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            desired_asset: None,
            denom_asset: "Dollah".to_owned(),
            exchanges: vec!["wyndex".to_string()]
        }
    );
    Ok(())
}

#[test]
fn test_update_config() -> anyhow::Result<()> {
    // Create the mock
    let mock = MockBech32::new("sender");

    // Set up the environment and contract
    let (account, abstr, app, wyndex) =
        setup(mock.clone(), Some(AssetEntry::new(wyndex_bundle::USD)))?;

    let new_target_currency = wyndex.eur_token.to_string();

    let dex_name = String::from("osmosis");

    abstr
        .ans_host
        .update_dexes(vec![dex_name.clone()], vec![])?;

    app.call_as(&account.address()?).update_config(
        Some("Ye-uh-roah".to_owned()),
        Clearable::new_opt(AssetEntry::new(&new_target_currency)),
        Some(vec![dex_name.clone()]),
    )?;

    let config = app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            desired_asset: Some(AssetEntry::new(wyndex_bundle::EUR)),
            denom_asset: "Ye-uh-roah".to_owned(),
            exchanges: vec![dex_name]
        }
    );
    Ok(())
}

#[test]
fn test_simple_tip() -> anyhow::Result<()> {
    let mock = MockBech32::new("sender");

    let (account, abstr_deployment, app, wyndex) = setup(mock.clone(), None)?;
    let mock: MockBech32 = abstr_deployment.ans_host.environment().clone();
    let WynDex {
        eur_token,
        usd_token: _,
        eur_usd_lp: _,
        ..
    } = wyndex;
    let tipper = mock.addr_make("tipper");
    let tip_amount = 100;
    let tip_currency = eur_token.to_string();
    let tip_coins = coins(tip_amount, tip_currency.clone());
    mock.set_balance(&tipper, tip_coins.clone())?;

    app.call_as(&tipper).tip(&tip_coins)?;

    let balance = mock.query_balance(&account.address()?, &tip_currency)?;
    assert_eq!(balance, Uint128::from(tip_amount));

    // Query tip count
    let tip_count_response: TipCountResponse = app.tip_count()?;
    assert_eq!(1, tip_count_response.count);

    // Query tipper
    let tipper_response: TipperResponse = app.tipper(tipper.to_string(), None, None, None)?;
    let expected_tipper = TipperResponse {
        address: tipper.clone(),
        tip_count: 1,
        total_amounts: vec![AnsAsset::new(wyndex_bundle::EUR, Uint128::new(100))],
    };
    assert_eq!(expected_tipper, tipper_response);

    // List tippers
    let tippers_response: TippersCountResponse = app.list_tippers_count(None, None)?;
    let tippers = tippers_response.tippers;
    assert_eq!(
        vec![TipperCountResponse {
            address: tipper.clone(),
            count: 1,
        }],
        tippers
    );

    // Tip amount at height
    mock.next_block()?;
    let current_block_height = mock.block_info()?.height;
    let tipper_response: TipperResponse =
        app.tipper(tipper.to_string(), Some(current_block_height), None, None)?;
    assert_eq!(
        tipper_response.total_amounts,
        vec![AnsAsset::new(
            AssetEntry::new(wyndex_bundle::EUR),
            Uint128::new(tip_amount)
        )]
    );

    let tipper_response: TipperResponse = app.tipper(
        tipper.to_string(),
        // previous block
        Some(current_block_height - 1),
        None,
        None,
    )?;
    assert_eq!(
        tipper_response.total_amounts,
        vec![AnsAsset::new(
            AssetEntry::new(wyndex_bundle::EUR),
            Uint128::zero()
        )]
    );

    Ok(())
}

#[test]
fn test_tip_swap() -> anyhow::Result<()> {
    let mock = MockBech32::new("sender");

    let (account, _abstr_deployment, app, wyndex) =
        setup(mock.clone(), Some(AssetEntry::new(wyndex_bundle::USD)))?;

    let WynDex {
        eur_token,
        usd_token,
        eur_usd_lp: _,
        ..
    } = wyndex;

    let tipper = mock.addr_make("tipper");
    let tip_amount = 100;
    let tip_currency = eur_token.to_string();
    let target_currency = usd_token.to_string();
    let tip_coins = coins(tip_amount, tip_currency.clone());
    mock.set_balance(&tipper, tip_coins.clone())?;

    app.call_as(&tipper).tip(&tip_coins)?;

    let balance = mock.query_balance(&account.address()?, &target_currency)?;
    assert!(!balance.is_zero());
    let balance = mock.query_balance(&account.address()?, &tip_currency)?;
    assert!(balance.is_zero());

    // Query tip count
    let tip_count_response: TipCountResponse = app.tip_count()?;
    assert_eq!(1, tip_count_response.count);

    // Query tipper
    let tipper_response: TipperResponse = app.tipper(tipper.to_string(), None, None, None)?;
    assert_eq!(1, tipper_response.total_amounts.len());

    // List tippers
    let tippers_response: TippersCountResponse = app.list_tippers_count(None, None)?;
    let tippers = tippers_response.tippers;
    assert_eq!(1, tippers.len());
    assert_eq!(
        TipperCountResponse {
            address: tipper,
            count: 1
        },
        tippers[0]
    );

    Ok(())
}

#[test]
fn test_tip_swap_and_not_swap() -> anyhow::Result<()> {
    let mock = MockBech32::new("sender");

    let (account, abstr_deployment, app, wyndex) =
        setup(mock.clone(), Some(AssetEntry::new(wyndex_bundle::USD)))?;

    let WynDex {
        eur_token,
        usd_token,
        eur_usd_lp: _,
        ..
    } = wyndex;

    let tipper = mock.addr_make("tipper");
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

    let balance = mock.query_balance(&account.address()?, &tip_currency)?;
    assert!(balance.is_zero());

    let balance = mock.query_balance(&account.address()?, &target_currency)?;
    assert!(!balance.is_zero());

    let balance = mock.query_balance(&account.address()?, tip_currency1)?;
    assert_eq!(balance, Uint128::from(tip_amount1));

    Ok(())
}

#[test]
fn test_cw20_tip() -> anyhow::Result<()> {
    // Create the mock
    let mock = MockBech32::new("sender");

    let (account, abstr_deployment, app, _wyndex) =
        setup(mock.clone(), Some(AssetEntry::new(wyndex_bundle::USD)))?;

    let tipper = mock.addr_make("tipper");
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
        to_json_binary("")?,
    )?;

    let tipper_balance = cw20_token.balance(tipper.to_string())?.balance;
    assert_eq!(starting_balance - tip_amount, tipper_balance.u128());

    let proxy_balance = cw20_token.balance(account.address()?.to_string())?.balance;
    assert_eq!(tip_amount, proxy_balance.u128());

    // Query tip count
    let tip_count_response: TipCountResponse = app.tip_count()?;
    assert_eq!(1, tip_count_response.count);

    // Query tipper
    let tipper_response: TipperResponse = app.tipper(tipper.to_string(), None, None, None)?;
    let expected_tipper = TipperResponse {
        address: tipper,
        tip_count: 1,
        total_amounts: vec![AnsAsset::new(cw20_token.addr_str()?, Uint128::new(100))],
    };
    assert_eq!(expected_tipper, tipper_response);

    Ok(())
}

#[test]
fn test_multiple_tippers() -> anyhow::Result<()> {
    // Create the mock
    let mock = MockBech32::new("sender");

    let (account, abstr_deployment, app, wyndex) = setup(mock, None)?;
    let mock: MockBech32 = abstr_deployment.ans_host.environment().clone();
    let WynDex {
        eur_token,
        usd_token: _,
        eur_usd_lp: _,
        ..
    } = wyndex;

    // First tipper
    let tipper1 = mock.addr_make("tipper1");
    let tip_amount1 = 100;
    let tip_currency = eur_token.to_string();
    let tip_coins = coins(tip_amount1, tip_currency.clone());
    mock.set_balance(&tipper1, tip_coins.clone())?;

    app.call_as(&tipper1).tip(&tip_coins)?;

    // Second tipper
    let tipper2 = mock.addr_make("tipper2");
    let tip_amount2 = 200;
    let tip_currency = eur_token.to_string();
    let tip_coins = coins(tip_amount2, tip_currency.clone());
    mock.set_balance(&tipper2, tip_coins.clone())?;

    app.call_as(&tipper2).tip(&tip_coins)?;

    let balance = mock.query_balance(&account.address()?, &tip_currency)?;
    assert_eq!(balance, Uint128::from(tip_amount1 + tip_amount2));

    // Query tip count
    let tip_count_response: TipCountResponse = app.tip_count()?;
    assert_eq!(2, tip_count_response.count);

    // Query first tipper
    let tipper_response1: TipperResponse = app.tipper(tipper1.to_string(), None, None, None)?;
    let expected_tipper = TipperResponse {
        address: tipper1.clone(),
        tip_count: 1,
        total_amounts: vec![AnsAsset::new(wyndex_bundle::EUR, Uint128::new(100))],
    };
    assert_eq!(expected_tipper, tipper_response1);

    // Query second tipper
    let tipper_response2: TipperResponse = app.tipper(tipper2.to_string(), None, None, None)?;
    let expected_tipper = TipperResponse {
        address: tipper2.clone(),
        tip_count: 1,
        total_amounts: vec![AnsAsset::new(wyndex_bundle::EUR, Uint128::new(200))],
    };
    assert_eq!(expected_tipper, tipper_response2);

    // List tippers
    let tippers_response: TippersCountResponse = app.list_tippers_count(None, None)?;
    let tippers = tippers_response.tippers;
    assert_eq!(
        vec![
            TipperCountResponse {
                address: tipper1,
                count: 1,
            },
            TipperCountResponse {
                address: tipper2,
                count: 1,
            }
        ],
        tippers
    );

    Ok(())
}

#[test]
fn test_sent_desired_asset() -> anyhow::Result<()> {
    // Create the mock
    let mock = MockBech32::new("sender");

    let (_, abstr_deployment, app, wyndex) =
        setup(mock, Some(AssetEntry::new(wyndex_bundle::USD)))?;
    let mock: MockBech32 = abstr_deployment.ans_host.environment().clone();
    let WynDex { usd_token, .. } = wyndex;

    let tipper = mock.addr_make("tipper1");
    let tip_amount = 100;
    let tip_currency = usd_token.to_string();
    let tip_coins = coins(tip_amount, tip_currency.clone());
    mock.set_balance(&tipper, tip_coins.clone())?;

    app.call_as(&tipper).tip(&tip_coins)?;

    // Query tipper
    let tipper_response: TipperResponse = app.tipper(tipper.to_string(), None, None, None)?;
    let expected_tipper = TipperResponse {
        address: tipper.clone(),
        tip_count: 1,
        total_amounts: vec![AnsAsset::new(wyndex_bundle::USD, Uint128::new(100))],
    };
    assert_eq!(expected_tipper, tipper_response);

    // Query tippers
    let tippers_response: TippersCountResponse = app.list_tippers_count(None, None)?;
    let expected_tipper_count = TipperCountResponse {
        address: tipper,
        count: 1,
    };
    assert_eq!(expected_tipper_count, tippers_response.tippers[0]);

    Ok(())
}
