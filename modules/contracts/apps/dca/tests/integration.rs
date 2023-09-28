mod common;
use std::cell::RefMut;

use abstract_core::objects::{
    dependency::DependencyResponse, module_version::ModuleDataResponse, AccountId, AssetEntry,
    PoolAddress, PoolReference, UncheckedContractEntry, UniquePoolId,
};
use abstract_core::AbstractError;
use abstract_core::{
    app::{BaseInstantiateMsg, BaseQueryMsgFns},
    objects::gov_type::GovernanceDetails,
};
use abstract_dex_adapter::interface::DexAdapter;
use abstract_dex_adapter::msg::{DexInstantiateMsg, OfferAsset};
use abstract_dex_adapter::DEX_ADAPTER_ID;
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns, *};
use dca_app::msg::{DCAResponse, Frequency};
use dca_app::state::{DCAEntry, DCAId};
use dca_app::{
    contract::{DCA_APP_ID, DCA_APP_VERSION},
    msg::{AppInstantiateMsg, ConfigResponse, InstantiateMsg},
    *,
};

use common::contracts;

use croncat_app::contract::CRONCAT_MODULE_VERSION;
use croncat_app::{contract::CRONCAT_ID, AppQueryMsgFns, CroncatApp, CRON_CAT_FACTORY};

use croncat_app::croncat_integration_utils::{AGENTS_NAME, MANAGER_NAME, TASKS_NAME};
use croncat_sdk_agents::msg::InstantiateMsg as AgentsInstantiateMsg;
use croncat_sdk_factory::msg::{
    ContractMetadataResponse, FactoryInstantiateMsg, FactoryQueryMsg, ModuleInstantiateInfo,
    VersionKind,
};
use croncat_sdk_manager::msg::ManagerInstantiateMsg;
use croncat_sdk_tasks::msg::TasksInstantiateMsg;

use cw20::Cw20Coin;

use cw_asset::AssetInfo;
use cw_orch::mock::cw_multi_test::{App, Executor};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{coin, coins, to_binary, Addr, Decimal, StdError, Uint128};
use wyndex_bundle::{WynDex, EUR, USD, WYNDEX as WYNDEX_WITHOUT_CHAIN};

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
// consts for testing
const ADMIN: &str = "admin";
const AGENT: &str = "agent";
const VERSION: &str = "1.0";
const DENOM: &str = "abstr";
const PAUSE_ADMIN: &str = "cosmos338dwgj5wm2tuahvfjdldz5s8hmt7l5aznw8jz9s2mmgj5c52jqgfq000";

fn setup_croncat_contracts(
    mut app: RefMut<App>,
    proxy_addr: String,
) -> anyhow::Result<(CronCatAddrs, Addr)> {
    let sender = Addr::unchecked(ADMIN);
    let pause_admin = Addr::unchecked(PAUSE_ADMIN);

    // Instantiate cw20
    let cw20_code_id = app.store_code(contracts::cw20_contract());
    let cw20_addr = app.instantiate_contract(
        cw20_code_id,
        sender.clone(),
        &cw20_base::msg::InstantiateMsg {
            name: "croncatcoins".to_owned(),
            symbol: "ccc".to_owned(),
            decimals: 6,
            initial_balances: vec![Cw20Coin {
                address: proxy_addr,
                amount: Uint128::new(105),
            }],
            mint: None,
            marketing: None,
        },
        &[],
        "cw20-contract".to_owned(),
        None,
    )?;

    let factory_code_id = app.store_code(contracts::croncat_factory_contract());
    let factory_addr = app.instantiate_contract(
        factory_code_id,
        sender.clone(),
        &FactoryInstantiateMsg {
            owner_addr: Some(sender.to_string()),
        },
        &[],
        "croncat-factory",
        None,
    )?;

    // Instantiate manager
    let code_id = app.store_code(contracts::croncat_manager_contract());
    let msg = ManagerInstantiateMsg {
        version: Some("1.0".to_owned()),
        croncat_tasks_key: (TASKS_NAME.to_owned(), [1, 0]),
        croncat_agents_key: (AGENTS_NAME.to_owned(), [1, 0]),
        pause_admin: pause_admin.clone(),
        gas_price: None,
        treasury_addr: None,
        cw20_whitelist: Some(vec![cw20_addr.to_string()]),
    };
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [1, 0],
        commit_id: "commit1".to_owned(),
        checksum: "checksum123".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&msg).unwrap(),
        contract_name: MANAGER_NAME.to_owned(),
    };
    app.execute_contract(
        sender.clone(),
        factory_addr.clone(),
        &croncat_factory::msg::ExecuteMsg::Deploy {
            kind: VersionKind::Agents,
            module_instantiate_info,
        },
        &[Coin {
            denom: DENOM.to_owned(),
            amount: Uint128::new(1),
        }],
    )
    .unwrap();

    // Instantiate agents
    let code_id = app.store_code(contracts::croncat_agents_contract());
    let msg = AgentsInstantiateMsg {
        version: Some(VERSION.to_owned()),
        croncat_manager_key: (MANAGER_NAME.to_owned(), [1, 0]),
        croncat_tasks_key: (TASKS_NAME.to_owned(), [1, 0]),
        pause_admin: pause_admin.clone(),
        agent_nomination_duration: None,
        min_tasks_per_agent: None,
        min_coins_for_agent_registration: None,
        agents_eject_threshold: None,
        min_active_agent_count: None,
        allowed_agents: Some(vec![AGENT.to_owned()]),
        public_registration: true,
    };
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [1, 0],
        commit_id: "commit123".to_owned(),
        checksum: "checksum321".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&msg).unwrap(),
        contract_name: AGENTS_NAME.to_owned(),
    };
    app.execute_contract(
        sender.clone(),
        factory_addr.to_owned(),
        &croncat_factory::msg::ExecuteMsg::Deploy {
            kind: VersionKind::Agents,
            module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    // Instantiate tasks
    let code_id = app.store_code(contracts::croncat_tasks_contract());
    let msg = TasksInstantiateMsg {
        version: Some(VERSION.to_owned()),
        chain_name: "atom".to_owned(),
        pause_admin,
        croncat_manager_key: (MANAGER_NAME.to_owned(), [1, 0]),
        croncat_agents_key: (AGENTS_NAME.to_owned(), [1, 0]),
        slot_granularity_time: None,
        gas_base_fee: None,
        gas_action_fee: None,
        gas_query_fee: None,
        gas_limit: None,
    };
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [1, 0],
        commit_id: "commit1".to_owned(),
        checksum: "checksum2".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_binary(&msg).unwrap(),
        contract_name: TASKS_NAME.to_owned(),
    };
    app.execute_contract(
        sender,
        factory_addr.to_owned(),
        &croncat_factory::msg::ExecuteMsg::Deploy {
            kind: VersionKind::Tasks,
            module_instantiate_info,
        },
        &[],
    )
    .unwrap();

    let metadata: ContractMetadataResponse = app
        .wrap()
        .query_wasm_smart(
            factory_addr.clone(),
            &croncat_sdk_factory::msg::FactoryQueryMsg::LatestContract {
                contract_name: MANAGER_NAME.to_owned(),
            },
        )
        .unwrap();
    let manager_address = metadata.metadata.unwrap().contract_addr;

    let metadata: ContractMetadataResponse = app
        .wrap()
        .query_wasm_smart(
            factory_addr.clone(),
            &croncat_sdk_factory::msg::FactoryQueryMsg::LatestContract {
                contract_name: TASKS_NAME.to_owned(),
            },
        )
        .unwrap();

    let tasks_address = metadata.metadata.unwrap().contract_addr;

    let response: ContractMetadataResponse = app.wrap().query_wasm_smart(
        &factory_addr,
        &FactoryQueryMsg::LatestContract {
            contract_name: AGENTS_NAME.to_string(),
        },
    )?;
    let agents_addr = response.metadata.unwrap().contract_addr;
    app.execute_contract(
        Addr::unchecked(AGENT),
        agents_addr.clone(),
        &croncat_sdk_agents::msg::ExecuteMsg::RegisterAgent {
            payable_account_id: None,
        },
        &[],
    )?;

    Ok((
        CronCatAddrs {
            factory: factory_addr,
            manager: manager_address,
            tasks: tasks_address,
            agents: agents_addr,
        },
        cw20_addr,
    ))
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
    let mock = Mock::new(&sender);

    // With funds
    mock.add_balance(&sender, coins(6_000_000_000, DENOM))?;
    mock.add_balance(&Addr::unchecked(AGENT), coins(6_000_000_000, DENOM))?;

    let (cron_cat_addrs, _proxy) =
        setup_croncat_contracts(mock.app.as_ref().borrow_mut(), sender.to_string())?;

    // Construct the DCA interface
    let mut dca_app = DCAApp::new(DCA_APP_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.to_string())?;
    abstr_deployment.ans_host.execute(
        &abstract_core::ans_host::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![("denom".to_owned(), AssetInfo::native(DENOM).into())],
            to_remove: vec![],
        },
        None,
    )?;
    // Deploy wyndex to the mock
    let wyndex = wyndex_bundle::WynDex::deploy_on(mock.clone(), Empty {})?;
    // Deploy dex adapter to the mock
    let dex_adapter = abstract_dex_adapter::interface::DexAdapter::new(DEX_ADAPTER_ID, mock.clone());

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
        .claim_namespace(AccountId::local(1), "croncat".to_string())?;
    cron_cat_app.deploy(croncat_app::contract::CRONCAT_MODULE_VERSION.parse()?)?;

    // Register factory entry
    let factory_entry = UncheckedContractEntry::try_from(CRON_CAT_FACTORY)?;
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
    account.manager.install_module(DEX_ADAPTER_ID, &Empty {}, None)?;
    let module_addr = account.manager.module_info(DEX_ADAPTER_ID)?.unwrap().address;
    dex_adapter.set_address(&module_addr);

    // Install croncat
    account.install_module(
        CRONCAT_ID,
        &croncat_app::msg::InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstr_deployment.ans_host.addr_str()?,
                version_control_address: abstr_deployment.version_control.addr_str()?,
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
                version_control_address: abstr_deployment.version_control.addr_str()?,
            },
            module: AppInstantiateMsg {
                native_asset: AssetEntry::new("denom"),
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
        DEX_ADAPTER_ID,
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
            native_asset: AssetEntry::from("denom"),
            dca_creation_amount: Uint128::new(5_000_000),
            refill_threshold: Uint128::new(1_000_000),
            max_spread: Decimal::percent(30),
        }
    );

    let module_data = apps.dca_app.module_data()?;
    assert_eq!(
        module_data,
        ModuleDataResponse {
            module_id: DCA_APP_ID.to_owned(),
            version: DCA_APP_VERSION.to_owned(),
            dependencies: vec![
                DependencyResponse {
                    id: CRONCAT_ID.to_owned(),
                    version_req: vec![format!("^{}", CRONCAT_MODULE_VERSION)]
                },
                DependencyResponse {
                    id: DEX_ADAPTER_ID.to_owned(),
                    version_req: vec![format!(
                        "^{}",
                        abstract_dex_adapter::contract::CONTRACT_VERSION.to_owned()
                    )]
                }
            ],
            metadata: None
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
    let dca = apps.dca_app.dca(DCAId(1))?;
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
    let dca = apps.dca_app.dca(DCAId(2))?;
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

    apps.dca_app.convert(DCAId(1))?;

    let usd_balance = mock.query_balance(&account.proxy.address()?, USD)?;
    assert_eq!(usd_balance, Uint128::new(98));
    let eur_balance = mock.query_balance(&account.proxy.address()?, EUR)?;
    assert_eq!(eur_balance, Uint128::new(9900));

    apps.dca_app.convert(DCAId(2))?;

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
                "asset pairing wyndex/usd,usd not found in ans_host"
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
    let err = apps.dca_app.convert(DCAId(1));
    assert_eq!(
        err.unwrap_err().root().to_string(),
        error::DCAError::NotManagerConvert {}.to_string()
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
        .task_info(apps.dca_app.addr_str()?, DCAId(1).into())?
        .task
        .unwrap()
        .task_hash;

    apps.dca_app.update_dca(
        DCAId(1),
        Some(WYNDEX_WITHOUT_CHAIN.into()),
        Some(Frequency::Cron("0 30 * * * *".to_string())),
        Some(OfferAsset::new(USD, 200_u128)),
        Some(EUR.into()),
    )?;

    let dca = apps.dca_app.dca(DCAId(1))?;
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
        .task_info(apps.dca_app.addr_str()?, DCAId(1).into())?
        .task
        .unwrap()
        .task_hash;

    assert_ne!(task_hash_before_update, task_hash_after_update);

    // Now without updating frequency
    apps.dca_app.update_dca(
        DCAId(1),
        None,
        None,
        Some(OfferAsset::new(USD, 250_u128)),
        None,
    )?;

    let dca = apps.dca_app.dca(DCAId(1))?;
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
        .task_info(apps.dca_app.addr_str()?, DCAId(1).into())?
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
    let err = apps
        .dca_app
        .update_dca(DCAId(1), Some("not_wyndex".into()), None, None, None);
    assert_querrier_err_eq(
        err.unwrap_err(),
        StdError::generic_err("DEX not_wyndex is not local to this network."),
    );

    // Not existing pair
    let err = apps.dca_app.update_dca(
        DCAId(1),
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
                "asset pairing wyndex/usd,usd not found in ans_host"
            ))
        )),
    );

    // Bad crontab string
    let err = apps.dca_app.update_dca(
        DCAId(1),
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

    apps.dca_app.cancel_dca(DCAId(1))?;

    let dca = apps.dca_app.dca(DCAId(1))?;
    assert_eq!(
        dca,
        DCAResponse {
            dca: None,
            pool_references: vec![]
        }
    );

    Ok(())
}
