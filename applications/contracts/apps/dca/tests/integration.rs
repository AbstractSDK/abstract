use std::cell::RefCell;
use std::rc::Rc;

use abstract_core::objects::{
    AssetEntry, PoolAddress, PoolReference, UncheckedContractEntry, UniquePoolId,
};
use abstract_core::AbstractError;
use abstract_core::{app::BaseInstantiateMsg, objects::gov_type::GovernanceDetails};
use abstract_dca_app::msg::{DCAResponse, Frequency};
use abstract_dca_app::state::{Config, DCAEntry};
use abstract_dca_app::{
    contract::{DCA_APP_ID, DCA_APP_VERSION},
    msg::{AppInstantiateMsg, ConfigResponse, InstantiateMsg},
    *,
};
use abstract_dex_adapter::interface::DexAdapter;
use abstract_dex_adapter::msg::{DexInstantiateMsg, OfferAsset};
use abstract_dex_adapter::EXCHANGE;
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns, *};
use croncat_app::{contract::CRONCAT_ID, AppQueryMsgFns, CroncatApp, CRON_CAT_FACTORY};
use croncat_integration_testing::test_helpers::set_up_croncat_contracts;
use croncat_integration_testing::DENOM;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{coin, Addr, Decimal, StdError, Uint128};
use wyndex_bundle::{WynDex, EUR, USD, WYNDEX as WYNDEX_WITHOUT_CHAIN};

// consts for testing
const ADMIN: &str = "admin";

#[allow(unused)]
struct CronCatAddrs {
    factory: Addr,
    manager: Addr,
    tasks: Addr,
    agents: Addr,
}

#[allow(unused)]
struct DeployedApps {
    dca_app: DCAApp<Mock>,
    dex_adapter: DexAdapter<Mock>,
    cron_cat_app: CroncatApp<Mock>,
    wyndex: WynDex,
}

/// Set up the test environment with the contract installed
#[allow(clippy::type_complexity)]
fn setup() -> anyhow::Result<(
    Mock,
    AbstractAccount<Mock>,
    Abstract<Mock>,
    DeployedApps,
    CronCatAddrs,
)> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mut mock = Mock::new(&sender);
    let cron_cat = set_up_croncat_contracts(None);
    mock.app = Rc::new(RefCell::new(cron_cat.app));
    let cron_cat_addrs = CronCatAddrs {
        factory: cron_cat.factory,
        manager: cron_cat.manager,
        tasks: cron_cat.tasks,
        agents: cron_cat.agents,
    };

    // Construct the DCA interface
    let mut dca_app = DCAApp::new(DCA_APP_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), Empty {})?;
    // Deploy wyndex to the mock
    let wyndex = wyndex_bundle::WynDex::deploy_on(mock.clone(), Empty {})?;
    // Deploy dex adapter to the mock
    let dex_adapter = abstract_dex_adapter::interface::DexAdapter::new(EXCHANGE, mock.clone());

    dex_adapter.deploy(
        abstract_dex_adapter::contract::CONTRACT_VERSION.parse()?,
        DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: 0,
        },
    )?;

    let mut cron_cat_app = CroncatApp::new(CRONCAT_ID, mock.clone());
    // Create account for croncat namespace
    abstr_deployment
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: ADMIN.to_string(),
        })?;
    abstr_deployment
        .version_control
        .claim_namespace(1, "croncat".to_string())?;
    cron_cat_app.deploy(croncat_app::contract::CRONCAT_MODULE_VERSION.parse()?)?;

    // Register factory entry
    let factory_entry = UncheckedContractEntry::try_from(CRON_CAT_FACTORY.to_owned())?;
    abstr_deployment.ans_host.execute(
        &abstract_core::ans_host::ExecuteMsg::UpdateContractAddresses {
            to_add: vec![(factory_entry, cron_cat_addrs.factory.to_string())],
            to_remove: vec![],
        },
        None,
    )?;
    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: ADMIN.to_string(),
            })?;
    // Install DEX
    account.manager.install_module(EXCHANGE, &Empty {}, None)?;
    let module_addr = account.manager.module_info(EXCHANGE)?.unwrap().address;
    dex_adapter.set_address(&module_addr);

    // Install croncat
    account.install_module(
        CRONCAT_ID,
        &croncat_app::msg::InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr_deployment.ans_host.addr_str()?,
            },
            module: croncat_app::msg::AppInstantiateMsg {},
        },
        None,
    )?;
    let module_addr = account.manager.module_info(CRONCAT_ID)?.unwrap().address;
    cron_cat_app.set_address(&module_addr);
    let manager_addr = account.manager.address()?;
    cron_cat_app.set_sender(&manager_addr);

    // Install DCA
    dca_app.deploy(DCA_APP_VERSION.parse()?)?;
    account.install_module(
        DCA_APP_ID,
        &InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr_deployment.ans_host.addr_str()?,
            },
            module: AppInstantiateMsg {
                native_denom: DENOM.to_owned(),
                dca_creation_amount: Uint128::new(5_000_000),
                refill_threshold: Uint128::new(1_000_000),
                max_spread: Decimal::percent(30),
            },
        },
        None,
    )?;

    let module_addr = account.manager.module_info(DCA_APP_ID)?.unwrap().address;
    dca_app.set_address(&module_addr);
    account.manager.update_adapter_authorized_addresses(
        EXCHANGE,
        vec![module_addr.to_string()],
        vec![],
    )?;

    dca_app.set_sender(&manager_addr);
    mock.set_balance(
        &account.proxy.address()?,
        vec![coin(50_000_000, DENOM), coin(10_000, EUR)],
    )?;

    let deployed_apps = DeployedApps {
        dca_app,
        dex_adapter,
        cron_cat_app,
        wyndex,
    };
    Ok((
        mock,
        account,
        abstr_deployment,
        deployed_apps,
        cron_cat_addrs,
    ))
}

fn assert_querrier_err_eq(left: CwOrchError, right: StdError) {
    let querier_contract_err =
        |err| StdError::generic_err(format!("Querier contract error: {}", err));
    assert_eq!(
        left.root().to_string(),
        querier_contract_err(right).to_string()
    )
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let (_mock, _account, _abstr, apps, _manager_addr) = setup()?;

    let config: ConfigResponse = apps.dca_app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            config: Config {
                native_denom: DENOM.to_owned(),
                dca_creation_amount: Uint128::new(5_000_000),
                refill_threshold: Uint128::new(1_000_000),
                max_spread: Decimal::percent(30),
            }
        }
    );
    Ok(())
}

#[test]
fn create_dca_convert() -> anyhow::Result<()> {
    let (mock, account, _abstr, mut apps, croncat_addrs) = setup()?;

    // create 2 dcas
    apps.dca_app.create_dca(
        WYNDEX_WITHOUT_CHAIN.to_owned(),
        Frequency::EveryNBlocks(1),
        OfferAsset::new(EUR, 100_u128),
        USD.into(),
    )?;
    apps.dca_app.create_dca(
        WYNDEX_WITHOUT_CHAIN.to_owned(),
        // HAPPY NEW YEAR :D
        Frequency::Cron("0 0 0 1 1 * *".to_owned()),
        OfferAsset::new(EUR, 250_u128),
        USD.into(),
    )?;

    // First dca
    let dca = apps.dca_app.dca("dca_1".to_owned())?;
    assert_eq!(
        dca,
        DCAResponse {
            dca: Some(DCAEntry {
                source_asset: OfferAsset::new(EUR, 100_u128),
                target_asset: USD.into(),
                frequency: Frequency::EveryNBlocks(1),
                dex: WYNDEX_WITHOUT_CHAIN.to_owned()
            }),
            pool_references: vec![PoolReference::new(
                UniquePoolId::new(1),
                PoolAddress::contract(apps.wyndex.eur_usd_pair.clone())
            )],
        }
    );

    // Second dca
    let dca = apps.dca_app.dca("dca_2".to_owned())?;
    assert_eq!(
        dca,
        DCAResponse {
            dca: Some(DCAEntry {
                source_asset: OfferAsset::new(EUR, 250_u128),
                target_asset: USD.into(),
                frequency: Frequency::Cron("0 0 0 1 1 * *".to_owned()),
                dex: WYNDEX_WITHOUT_CHAIN.to_owned()
            }),
            pool_references: vec![PoolReference::new(
                UniquePoolId::new(1),
                PoolAddress::contract(apps.wyndex.eur_usd_pair)
            )],
        }
    );

    // Only manager should be able to execute this one
    apps.dca_app.set_sender(&croncat_addrs.manager);

    apps.dca_app.convert("dca_1".to_owned())?;

    let usd_balance = mock.query_balance(&account.proxy.address()?, USD)?;
    assert_eq!(usd_balance, Uint128::new(98));
    let eur_balance = mock.query_balance(&account.proxy.address()?, EUR)?;
    assert_eq!(eur_balance, Uint128::new(9900));

    apps.dca_app.convert("dca_2".to_owned())?;

    let usd_balance = mock.query_balance(&account.proxy.address()?, USD)?;
    assert_eq!(usd_balance, Uint128::new(335));
    let eur_balance = mock.query_balance(&account.proxy.address()?, EUR)?;
    assert_eq!(eur_balance, Uint128::new(9650));

    Ok(())
}

#[test]
fn create_dca_convert_negative() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps, _croncat_addrs) = setup()?;

    // Not existing DEX
    let err = apps.dca_app.create_dca(
        "not_wyndex".to_owned(),
        Frequency::EveryNBlocks(1),
        OfferAsset::new(EUR, 100_u128),
        USD.into(),
    );
    assert_querrier_err_eq(
        err.unwrap_err(),
        StdError::generic_err("DEX not_wyndex is not local to this network."),
    );

    // Not existing pair
    let err = apps.dca_app.create_dca(
        WYNDEX_WITHOUT_CHAIN.to_owned(),
        Frequency::EveryNBlocks(1),
        OfferAsset::new(USD, 100_u128),
        USD.into(),
    );
    assert_querrier_err_eq(
        err.unwrap_err(),
        StdError::generic_err(format!(
            "Failed to get pair address for {offer_asset:?} and {target_asset:?}: {e}",
            offer_asset = OfferAsset::new(USD, 100_u128),
            target_asset = AssetEntry::new(USD),
            e = AbstractError::from(StdError::generic_err(
                "asset pairing wynd/usd,usd not found in ans_host"
            ))
        )),
    );

    // Bad crontab string
    let err = apps.dca_app.create_dca(
        WYNDEX_WITHOUT_CHAIN.to_owned(),
        Frequency::Cron("bad cron".to_owned()),
        OfferAsset::new(USD, 100_u128),
        EUR.into(),
    );
    assert_eq!(err.unwrap_err().root().to_string(), "Invalid interval");

    apps.dca_app.create_dca(
        WYNDEX_WITHOUT_CHAIN.to_owned(),
        Frequency::EveryNBlocks(1),
        OfferAsset::new(EUR, 100_u128),
        USD.into(),
    )?;

    // Only manager should be able to execute this one
    let err = apps.dca_app.convert("dca_1".to_owned());
    assert_eq!(
        err.unwrap_err().root().to_string(),
        error::AppError::NotManagerConvert {}.to_string()
    );
    Ok(())
}

#[test]
fn update_dca() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps, _croncat_addrs) = setup()?;

    // create dca
    apps.dca_app.create_dca(
        WYNDEX_WITHOUT_CHAIN.to_owned(),
        Frequency::EveryNBlocks(1),
        OfferAsset::new(EUR, 150_u128),
        USD.into(),
    )?;

    let task_hash_before_update = apps
        .cron_cat_app
        .task_info(apps.dca_app.addr_str()?, "dca_1".to_owned())?
        .task
        .unwrap()
        .task_hash;

    apps.dca_app.update_dca(
        "dca_1".to_owned(),
        Some(WYNDEX_WITHOUT_CHAIN.into()),
        Some(Frequency::Cron("0 30 * * * *".to_string())),
        Some(OfferAsset::new(USD, 200_u128)),
        Some(EUR.into()),
    )?;

    let dca = apps.dca_app.dca("dca_1".to_owned())?;
    assert_eq!(
        dca,
        DCAResponse {
            dca: Some(DCAEntry {
                source_asset: OfferAsset::new(USD, 200_u128),
                target_asset: EUR.into(),
                frequency: Frequency::Cron("0 30 * * * *".to_string()),
                dex: WYNDEX_WITHOUT_CHAIN.to_owned()
            }),
            pool_references: vec![PoolReference::new(
                UniquePoolId::new(1),
                PoolAddress::contract(apps.wyndex.eur_usd_pair.clone())
            )],
        }
    );

    let task_hash_after_update = apps
        .cron_cat_app
        .task_info(apps.dca_app.addr_str()?, "dca_1".to_owned())?
        .task
        .unwrap()
        .task_hash;

    assert_ne!(task_hash_before_update, task_hash_after_update);

    // Now without updating frequency
    apps.dca_app.update_dca(
        "dca_1".to_owned(),
        None,
        None,
        Some(OfferAsset::new(USD, 250_u128)),
        None,
    )?;

    let dca = apps.dca_app.dca("dca_1".to_owned())?;
    assert_eq!(
        dca,
        DCAResponse {
            dca: Some(DCAEntry {
                source_asset: OfferAsset::new(USD, 250_u128),
                target_asset: AssetEntry::new(EUR),
                frequency: Frequency::Cron("0 30 * * * *".to_string()),
                dex: WYNDEX_WITHOUT_CHAIN.to_owned()
            }),
            pool_references: vec![PoolReference::new(
                UniquePoolId::new(1),
                PoolAddress::contract(apps.wyndex.eur_usd_pair)
            )],
        }
    );

    let task_hash_after_second_update = apps
        .cron_cat_app
        .task_info(apps.dca_app.addr_str()?, "dca_1".to_owned())?
        .task
        .unwrap()
        .task_hash;

    assert_eq!(task_hash_after_update, task_hash_after_second_update);

    Ok(())
}

#[test]
fn update_dca_negative() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps, _croncat_addrs) = setup()?;

    // create dca
    apps.dca_app.create_dca(
        WYNDEX_WITHOUT_CHAIN.to_owned(),
        Frequency::EveryNBlocks(1),
        OfferAsset::new(EUR, 150_u128),
        USD.into(),
    )?;

    // Not existing dex
    let err = apps.dca_app.update_dca(
        "dca_1".to_owned(),
        Some("not_wyndex".into()),
        None,
        None,
        None,
    );
    assert_querrier_err_eq(
        err.unwrap_err(),
        StdError::generic_err("DEX not_wyndex is not local to this network."),
    );

    // Not existing pair
    let err = apps.dca_app.update_dca(
        "dca_1".to_owned(),
        None,
        None,
        Some(OfferAsset::new(USD, 200_u128)),
        Some(USD.into()),
    );

    assert_querrier_err_eq(
        err.unwrap_err(),
        StdError::generic_err(format!(
            "Failed to get pair address for {offer_asset:?} and {target_asset:?}: {e}",
            offer_asset = OfferAsset::new(USD, 200_u128),
            target_asset = AssetEntry::new(USD),
            e = AbstractError::from(StdError::generic_err(
                "asset pairing wynd/usd,usd not found in ans_host"
            ))
        )),
    );

    // Bad crontab string
    let err = apps.dca_app.update_dca(
        "dca_1".to_owned(),
        None,
        Some(Frequency::Cron("bad cron".to_owned())),
        None,
        None,
    );
    assert_eq!(err.unwrap_err().root().to_string(), "Invalid interval");

    Ok(())
}

#[test]
fn cancel_dca() -> anyhow::Result<()> {
    let (_mock, _account, _abstr, apps, _croncat_addrs) = setup()?;

    // create dca
    apps.dca_app.create_dca(
        WYNDEX_WITHOUT_CHAIN.to_owned(),
        Frequency::EveryNBlocks(1),
        OfferAsset::new(EUR, 100_u128),
        USD.into(),
    )?;

    apps.dca_app.cancel_dca("dca_1".to_owned())?;

    let dca = apps.dca_app.dca("dca_1".to_owned())?;
    assert_eq!(
        dca,
        DCAResponse {
            dca: None,
            pool_references: vec![]
        }
    );

    Ok(())
}
