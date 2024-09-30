mod common;

use abstract_app::std::{
    ans_host::ContractsResponse,
    objects::{
        account::AccountTrace, gov_type::GovernanceDetails, AccountId, UncheckedContractEntry,
    },
};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, DeployStrategy, RegistryExecFns};
use common::contracts;
use cosmwasm_std::{coins, to_json_binary, BankMsg, Uint128, WasmMsg};
use croncat_app::{
    contract::{CRONCAT_ID, CRONCAT_MODULE_VERSION},
    error::AppError,
    msg::{ActiveTasksByCreatorResponse, ActiveTasksResponse, AppInstantiateMsg, ConfigResponse},
    state::Config,
    AppExecuteMsgFns, AppQueryMsgFns, Croncat, CRON_CAT_FACTORY,
};
use croncat_integration_utils::{AGENTS_NAME, MANAGER_NAME, TASKS_NAME};
use croncat_sdk_agents::msg::InstantiateMsg as AgentsInstantiateMsg;
use croncat_sdk_factory::msg::{
    ContractMetadataResponse, FactoryInstantiateMsg, FactoryQueryMsg, ModuleInstantiateInfo,
    VersionKind,
};
use croncat_sdk_manager::{
    msg::{ManagerExecuteMsg, ManagerInstantiateMsg},
    types::{TaskBalance, TaskBalanceResponse},
};
use croncat_sdk_tasks::{
    msg::TasksInstantiateMsg,
    types::{Action, TaskRequest, TaskResponse},
};
use cw20::{Cw20Coin, Cw20CoinVerified, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_asset::{Asset, AssetList, AssetListUnchecked};
use cw_orch::mock::cw_multi_test::Executor;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

use crate::common::contracts::TasksResponseCaster;
// consts for testing
const AGENT: &str = "agent";
const VERSION: &str = "1.0";
const DENOM: &str = "abstr";
const PAUSE_ADMIN: &str = "cosmos338dwgj5wm2tuahvfjdldz5s8hmt7l5aznw8jz9s2mmgj5c52jqgfq000";

fn setup_croncat_contracts(mock: MockBech32, proxy_addr: String) -> anyhow::Result<(Addr, Addr)> {
    let sender = mock.sender_addr();
    let pause_admin = Addr::unchecked(PAUSE_ADMIN);
    let agent_addr = mock.addr_make(AGENT);

    // Instantiate cw20
    let mut app = mock.app.borrow_mut();
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
        msg: to_json_binary(&msg).unwrap(),
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
        allowed_agents: Some(vec![agent_addr.to_string()]),
        public_registration: true,
    };
    let module_instantiate_info = ModuleInstantiateInfo {
        code_id,
        version: [1, 0],
        commit_id: "commit123".to_owned(),
        checksum: "checksum321".to_owned(),
        changelog_url: None,
        schema: None,
        msg: to_json_binary(&msg).unwrap(),
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
        msg: to_json_binary(&msg).unwrap(),
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

    let response: ContractMetadataResponse = app.wrap().query_wasm_smart(
        &factory_addr,
        &FactoryQueryMsg::LatestContract {
            contract_name: AGENTS_NAME.to_string(),
        },
    )?;
    let agents_addr = response.metadata.unwrap().contract_addr;
    app.execute_contract(
        agent_addr,
        agents_addr,
        &croncat_sdk_agents::msg::ExecuteMsg::RegisterAgent {
            payable_account_id: None,
        },
        &[],
    )?;

    Ok((factory_addr, cw20_addr))
}

struct TestingSetup {
    account: AbstractAccount<MockBech32>,
    #[allow(unused)]
    abstr_deployment: Abstract<MockBech32>,
    module_contract: Croncat<MockBech32>,
    cw20_addr: Addr,
    mock: MockBech32,
}

/// Set up the test environment with the contract installed
fn setup() -> anyhow::Result<TestingSetup> {
    // Create the mock
    let mock = MockBech32::new("mock");
    let sender = mock.sender_addr();

    mock.set_balance(&mock.addr_make(AGENT), coins(500_000, DENOM))?;
    // Construct the counter interface
    let mut contract = Croncat::new(CRONCAT_ID, mock.clone());
    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.to_string())?;
    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: mock.sender_addr().to_string(),
            })?;
    // claim the namespace so app can be deployed
    abstr_deployment.registry.claim_namespace(
        AccountId::new(1, AccountTrace::Local)?,
        "croncat".to_owned(),
    )?;

    // Instantiating croncat contracts
    mock.set_balance(&sender, coins(100, DENOM))?;
    let (factory_addr, cw20_addr) =
        setup_croncat_contracts(mock.clone(), account.proxy.addr_str()?)?;

    let factory_entry = UncheckedContractEntry::try_from(CRON_CAT_FACTORY)?;
    abstr_deployment.ans_host.execute(
        &abstract_app::std::ans_host::ExecuteMsg::UpdateContractAddresses {
            to_add: vec![(factory_entry, factory_addr.to_string())],
            to_remove: vec![],
        },
        None,
    )?;

    contract.deploy(CRONCAT_MODULE_VERSION.parse()?, DeployStrategy::Try)?;
    account.install_app(&contract, &AppInstantiateMsg {}, None)?;

    let manager_addr = account.address()?;
    contract.set_sender(&manager_addr);
    mock.set_balance(&account.address()?, coins(500_000, DENOM))?;

    Ok(TestingSetup {
        account,
        abstr_deployment,
        module_contract: contract,
        cw20_addr,
        mock,
    })
}

#[test]
fn all_in_one() -> anyhow::Result<()> {
    // Set up the environment and contract
    let TestingSetup {
        account,
        module_contract,
        cw20_addr,
        mock,
        ..
    } = setup()?;

    let cw20_amount = Cw20Coin {
        address: cw20_addr.to_string(),
        amount: Uint128::new(100),
    };
    let task = TaskRequest {
        interval: croncat_sdk_tasks::types::Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![
            Action {
                msg: BankMsg::Send {
                    to_address: mock.addr_make("receiver").to_string(),
                    amount: coins(1, DENOM),
                }
                .into(),
                gas_limit: None,
            },
            Action {
                msg: WasmMsg::Execute {
                    contract_addr: cw20_addr.to_string(),
                    msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: mock.addr_make("bob").to_string(),
                        amount: Uint128::new(100),
                    })?,
                    funds: vec![],
                }
                .into(),
                gas_limit: Some(120),
            },
        ],
        queries: None,
        transforms: None,
        cw20: Some(cw20_amount.clone()),
    };

    // Task creation
    let assets = {
        let mut assets = AssetList::from(coins(45_000, DENOM));
        assets.add(&Asset::cw20(cw20_addr.clone(), cw20_amount.amount))?;
        AssetListUnchecked::from(assets)
    };
    let task_tag = "test_sends".to_owned();
    module_contract.create_task(assets, Box::new(task), task_tag)?;

    let active_tasks_response: ActiveTasksResponse =
        module_contract.active_tasks(None, None, None)?;
    let active_tasks = active_tasks_response.unchecked();
    assert_eq!(active_tasks.len(), 1);

    let active_tasks_by_creator_response: ActiveTasksByCreatorResponse =
        module_contract.active_tasks_by_creator(account.manager.addr_str()?, None, None, None)?;
    let active_tasks_by_creator = active_tasks_by_creator_response.unchecked();
    assert_eq!(active_tasks_by_creator.len(), 1);

    // Refilling task
    let task_balance1: TaskBalance = module_contract
        .task_balance(active_tasks[0].0.to_string(), active_tasks[0].1.clone())?
        .balance
        .unwrap();
    let assets = {
        let mut assets = AssetList::from(coins(100, DENOM));
        assets.add(&Asset::cw20(cw20_addr.clone(), Uint128::new(5)))?;
        AssetListUnchecked::from(assets)
    };
    module_contract.refill_task(assets, active_tasks[0].1.clone())?;
    let task_balance2: TaskBalance = module_contract
        .task_balance(active_tasks[0].0.to_string(), active_tasks[0].1.clone())?
        .balance
        .unwrap();
    assert_eq!(
        task_balance2.native_balance,
        task_balance1.native_balance + Uint128::new(100)
    );
    assert_eq!(
        task_balance2.cw20_balance.unwrap().amount,
        task_balance1.cw20_balance.unwrap().amount + Uint128::new(5)
    );

    // Removing a task

    // Check that module balance is empty before remove
    let module_balance = mock.query_balance(&module_contract.address()?, DENOM)?;
    assert!(module_balance.is_zero());
    let module_cw20_balance: cw20::BalanceResponse = mock.query(
        &Cw20QueryMsg::Balance {
            address: module_contract.addr_str()?,
        },
        &cw20_addr,
    )?;
    assert!(module_cw20_balance.balance.is_zero());

    // Saving current proxy balances to check balance changes
    let proxy_balance1 = mock.query_balance(&account.address()?, DENOM)?;
    let proxy_cw20_balance1: cw20::BalanceResponse = mock.query(
        &Cw20QueryMsg::Balance {
            address: account.proxy.addr_str()?,
        },
        &cw20_addr,
    )?;

    // Module balance is zero
    let module_balance = mock.query_balance(&module_contract.address()?, DENOM)?;
    assert!(module_balance.is_zero());
    let manager_cw20_balance: cw20::BalanceResponse = mock.query(
        &Cw20QueryMsg::Balance {
            address: module_contract.addr_str()?,
        },
        &cw20_addr,
    )?;
    assert!(manager_cw20_balance.balance.is_zero());

    module_contract.remove_task(active_tasks[0].1.clone())?;

    // After task is removed check all balances got not here
    let module_balance = mock.query_balance(&module_contract.address()?, DENOM)?;
    assert!(module_balance.is_zero());

    let module_cw20_balance: cw20::BalanceResponse = mock.query(
        &Cw20QueryMsg::Balance {
            address: module_contract.addr_str()?,
        },
        &cw20_addr,
    )?;
    assert!(module_cw20_balance.balance.is_zero());

    // Everything landed on proxy contract
    let proxy_balance2 = mock.query_balance(&account.address()?, DENOM)?;
    assert_eq!(proxy_balance2, proxy_balance1 + Uint128::new(45_100));
    let proxy_cw20_balance2: cw20::BalanceResponse = mock.query(
        &Cw20QueryMsg::Balance {
            address: account.proxy.addr_str()?,
        },
        &cw20_addr,
    )?;
    assert_eq!(
        proxy_cw20_balance2.balance,
        proxy_cw20_balance1.balance + Uint128::new(105)
    );

    // State updated
    let active_tasks_response: ActiveTasksResponse =
        module_contract.active_tasks(None, None, None)?;
    let active_tasks = active_tasks_response.unchecked();
    assert_eq!(active_tasks.len(), 0);

    Ok(())
}

#[test]
fn admin() -> anyhow::Result<()> {
    // Set up the environment and contract
    let TestingSetup {
        mut module_contract,
        mock,
        cw20_addr,
        ..
    } = setup()?;

    let task = TaskRequest {
        interval: croncat_sdk_tasks::types::Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: BankMsg::Send {
                to_address: mock.addr_make("receiver").to_string(),
                amount: coins(1, DENOM),
            }
            .into(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20: None,
    };

    // Not admin sender
    // Neither installed module
    module_contract.set_sender(&cw20_addr);

    let expected_err = abstract_app::sdk::AbstractSdkError::MissingModule {
        module: "crates.io:cw20-base".to_owned(),
    }
    .to_string();

    let err = module_contract.update_config();
    assert_eq!(
        err.unwrap_err().root().to_string(),
        cw_controllers::AdminError::NotAdmin {}.to_string()
    );
    let task_tag = "test_tag".to_owned();
    let err = module_contract.create_task(AssetListUnchecked::default(), Box::new(task), task_tag);
    assert_eq!(err.unwrap_err().root().to_string(), expected_err);

    let err = module_contract.remove_task("aloha:321".to_owned());
    assert_eq!(err.unwrap_err().root().to_string(), expected_err);

    let err = module_contract.refill_task(AssetListUnchecked::default(), "woof:123".to_owned());
    assert_eq!(err.unwrap_err().root().to_string(), expected_err);

    Ok(())
}

#[test]
fn update_config() -> anyhow::Result<()> {
    // Set up the environment and contract
    let TestingSetup {
        module_contract, ..
    } = setup()?;

    let config_res: ConfigResponse = module_contract.config()?;

    assert_eq!(config_res.config, Config {});

    module_contract.update_config()?;

    let config_res: ConfigResponse = module_contract.config()?;
    assert_eq!(config_res.config, Config {});
    Ok(())
}

#[test]
fn create_task() -> anyhow::Result<()> {
    // Set up the environment and contract
    let TestingSetup {
        module_contract,
        mock,
        account,
        cw20_addr,
        ..
    } = setup()?;

    // Task without any cw20s
    let task = TaskRequest {
        interval: croncat_sdk_tasks::types::Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: BankMsg::Send {
                to_address: mock.addr_make("receiver").to_string(),
                amount: coins(1, DENOM),
            }
            .into(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20: None,
    };
    let task_tag = "test_tag".to_owned();
    let assets = AssetListUnchecked::from(AssetList::from(coins(45_000, DENOM)));
    module_contract.create_task(assets, Box::new(task), task_tag.clone())?;

    let active_tasks_response: ActiveTasksResponse =
        module_contract.active_tasks(None, None, None)?;
    let active_tasks = active_tasks_response.unchecked();
    assert_eq!(active_tasks.len(), 1);

    let task_info_response: TaskResponse =
        module_contract.task_info(active_tasks[0].0.to_string(), active_tasks[0].1.to_string())?;
    assert_eq!(
        task_info_response.task.unwrap().owner_addr,
        account.proxy.addr_str()?
    );

    // Task with some cw20s
    let cw20_amount = Cw20Coin {
        address: cw20_addr.to_string(),
        amount: Uint128::new(20),
    };
    let task = TaskRequest {
        interval: croncat_sdk_tasks::types::Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: mock.addr_make("bob").to_string(),
                    amount: Uint128::new(20),
                })?,
                funds: vec![],
            }
            .into(),
            gas_limit: Some(120),
        }],
        queries: None,
        transforms: None,
        cw20: Some(cw20_amount.clone()),
    };

    // Let's attach now 2x of the cw20s and create two tasks LOL
    let assets = {
        let mut assets = AssetList::from(coins(40_000, DENOM));
        assets.add(&Asset::cw20(cw20_addr.clone(), Uint128::new(40)))?;
        AssetListUnchecked::from(assets)
    };
    let err = module_contract.create_task(assets.clone(), Box::new(task.clone()), task_tag.clone());
    assert_eq!(
        err.unwrap_err().root().to_string(),
        AppError::TaskAlreadyExists { task_tag }.to_string()
    );
    let task_tag = "test_tag2".to_owned();
    module_contract.create_task(assets, Box::new(task), task_tag)?;

    let active_tasks_response: ActiveTasksResponse =
        module_contract.active_tasks(None, None, None)?;
    let active_tasks = active_tasks_response.unchecked();
    assert_eq!(active_tasks.len(), 2);

    // This task creation we won't attach cw20s because we had some unused balance from the last creation
    let task = TaskRequest {
        interval: croncat_sdk_tasks::types::Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: mock.addr_make("alice").to_string(),
                    amount: Uint128::new(20),
                })?,
                funds: vec![],
            }
            .into(),
            gas_limit: Some(120),
        }],
        queries: None,
        transforms: None,
        cw20: Some(cw20_amount),
    };
    let task_tag = "test_tag3".to_owned();
    let assets = AssetListUnchecked::from(AssetList::from(coins(45_000, DENOM)));
    module_contract.create_task(assets, Box::new(task), task_tag)?;

    let active_tasks_response: ActiveTasksResponse =
        module_contract.active_tasks(None, None, None)?;
    let active_tasks = active_tasks_response.unchecked();
    assert_eq!(active_tasks.len(), 3);

    Ok(())
}

#[test]
fn refill_task() -> anyhow::Result<()> {
    // Set up the environment and contract
    let TestingSetup {
        module_contract,
        mock,
        account: _,
        cw20_addr,
        ..
    } = setup()?;

    let cw20_amount = Cw20Coin {
        address: cw20_addr.to_string(),
        amount: Uint128::new(20),
    };
    let task = TaskRequest {
        interval: croncat_sdk_tasks::types::Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: mock.addr_make("bob").to_string(),
                    amount: Uint128::new(20),
                })?,
                funds: vec![],
            }
            .into(),
            gas_limit: Some(120),
        }],
        queries: None,
        transforms: None,
        cw20: Some(cw20_amount),
    };
    let task_tag = "test_tag".to_owned();
    let assets = {
        let mut assets = AssetList::from(coins(40_000, DENOM));
        assets.add(&Asset::cw20(cw20_addr.clone(), Uint128::new(20)))?;
        AssetListUnchecked::from(assets)
    };
    module_contract.create_task(assets, Box::new(task), task_tag)?;

    let active_tasks_response: ActiveTasksResponse =
        module_contract.active_tasks(None, None, None)?;
    let active_tasks = active_tasks_response.unchecked();
    let (creator_addr, task_tag) = active_tasks[0].clone();

    let task_balance: TaskBalanceResponse =
        module_contract.task_balance(creator_addr.to_string(), task_tag.clone())?;
    assert_eq!(
        task_balance.balance.unwrap(),
        TaskBalance {
            native_balance: Uint128::new(40_000),
            cw20_balance: Some(Cw20CoinVerified {
                address: cw20_addr.clone(),
                amount: Uint128::new(20)
            }),
            ibc_balance: None
        }
    );

    // Refill only with native coins
    let assets = AssetListUnchecked::from(AssetList::from(coins(123, DENOM)));
    module_contract.refill_task(assets, task_tag.clone())?;
    let task_balance: TaskBalanceResponse =
        module_contract.task_balance(creator_addr.to_string(), task_tag.clone())?;
    assert_eq!(
        task_balance.balance.unwrap(),
        TaskBalance {
            native_balance: Uint128::new(40_123),
            cw20_balance: Some(Cw20CoinVerified {
                address: cw20_addr.clone(),
                amount: Uint128::new(20)
            }),
            ibc_balance: None
        }
    );

    // Refill only with cw20 coins
    let assets = {
        let mut assets = AssetList::new();
        assets.add(&Asset::cw20(cw20_addr.clone(), Uint128::new(25)))?;
        AssetListUnchecked::from(assets)
    };
    module_contract.refill_task(assets, task_tag.clone())?;
    let task_balance: TaskBalanceResponse =
        module_contract.task_balance(creator_addr.to_string(), task_tag.clone())?;
    assert_eq!(
        task_balance.balance.unwrap(),
        TaskBalance {
            native_balance: Uint128::new(40_123),
            cw20_balance: Some(Cw20CoinVerified {
                address: cw20_addr.clone(),
                amount: Uint128::new(45)
            }),
            ibc_balance: None
        }
    );

    // Refill with both
    let assets = {
        let mut assets = AssetList::from(coins(1_000, DENOM));
        assets.add(&Asset::cw20(cw20_addr.clone(), Uint128::new(55)))?;
        AssetListUnchecked::from(assets)
    };
    module_contract.refill_task(assets, task_tag.clone())?;
    let task_balance: TaskBalanceResponse =
        module_contract.task_balance(creator_addr.to_string(), task_tag)?;
    assert_eq!(
        task_balance.balance.unwrap(),
        TaskBalance {
            native_balance: Uint128::new(41_123),
            cw20_balance: Some(Cw20CoinVerified {
                address: cw20_addr,
                amount: Uint128::new(100)
            }),
            ibc_balance: None
        }
    );

    Ok(())
}

#[test]
fn remove_task() -> anyhow::Result<()> {
    // Set up the environment and contract
    let TestingSetup {
        module_contract,
        mock,
        account,
        cw20_addr,
        abstr_deployment,
        ..
    } = setup()?;

    // Create two tasks
    let cw20_amount = Cw20Coin {
        address: cw20_addr.to_string(),
        amount: Uint128::new(30),
    };
    let task = TaskRequest {
        interval: croncat_sdk_tasks::types::Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: mock.addr_make("bob").to_string(),
                    amount: Uint128::new(20),
                })?,
                funds: vec![],
            }
            .into(),
            gas_limit: Some(120),
        }],
        queries: None,
        transforms: None,
        cw20: Some(cw20_amount),
    };
    let task_tag1 = "test_tag1".to_owned();
    let assets = {
        let mut assets = AssetList::from(coins(40_000, DENOM));
        assets.add(&Asset::cw20(cw20_addr.clone(), Uint128::new(30)))?;
        AssetListUnchecked::from(assets)
    };
    module_contract.create_task(assets, Box::new(task), task_tag1)?;

    let cw20_amount = Cw20Coin {
        address: cw20_addr.to_string(),
        amount: Uint128::new(40),
    };
    let task = TaskRequest {
        interval: croncat_sdk_tasks::types::Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: WasmMsg::Execute {
                contract_addr: cw20_addr.to_string(),
                msg: to_json_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: mock.addr_make("alice").to_string(),
                    amount: Uint128::new(30),
                })?,
                funds: vec![],
            }
            .into(),
            gas_limit: Some(120),
        }],
        queries: None,
        transforms: None,
        cw20: Some(cw20_amount),
    };
    let task_tag2 = "test_tag2".to_owned();
    let assets = {
        let mut assets = AssetList::from(coins(40_000, DENOM));
        assets.add(&Asset::cw20(cw20_addr.clone(), Uint128::new(40)))?;
        AssetListUnchecked::from(assets)
    };
    module_contract.create_task(assets, Box::new(task), task_tag2)?;

    // One of them will be removed by the agent
    {
        mock.wait_blocks(3)?;
        let contracts_response: ContractsResponse =
            abstr_deployment
                .ans_host
                .query(&abstract_app::std::ans_host::QueryMsg::Contracts {
                    entries: vec![UncheckedContractEntry::try_from(CRON_CAT_FACTORY)?.into()],
                })?;
        let factory_addr: Addr = contracts_response.contracts[0].1.clone();
        let response: ContractMetadataResponse = mock.query(
            &FactoryQueryMsg::LatestContract {
                contract_name: MANAGER_NAME.to_string(),
            },
            &factory_addr,
        )?;
        let manager_addr: Addr = response.metadata.unwrap().contract_addr;
        let agent = mock.addr_make(AGENT);
        mock.app.borrow_mut().execute_contract(
            agent,
            manager_addr,
            &ManagerExecuteMsg::ProxyCall { task_hash: None },
            &[],
        )?;
    };

    // Note: not updated
    let active_tasks_response: ActiveTasksResponse =
        module_contract.active_tasks(None, None, None)?;
    let active_tasks = active_tasks_response.unchecked();
    assert_eq!(active_tasks.len(), 2);

    // Updated here
    let active_tasks_checked_response: ActiveTasksResponse =
        module_contract.active_tasks(Some(true), None, None)?;
    let (mut scheduled_tasks, mut removed_tasks) = active_tasks_checked_response.checked();

    assert_eq!(scheduled_tasks.len(), 1);
    assert_eq!(removed_tasks.len(), 1);

    let (active_task, not_active_task) = (
        scheduled_tasks.pop().unwrap().1,
        removed_tasks.pop().unwrap().1,
    );

    let proxy_cw20_balance1: cw20::BalanceResponse = mock.query(
        &Cw20QueryMsg::Balance {
            address: account.proxy.addr_str()?,
        },
        &cw20_addr,
    )?;

    module_contract.remove_task(not_active_task)?;

    let proxy_cw20_balance2: cw20::BalanceResponse = mock.query(
        &Cw20QueryMsg::Balance {
            address: account.proxy.addr_str()?,
        },
        &cw20_addr,
    )?;

    assert!(proxy_cw20_balance2.balance > proxy_cw20_balance1.balance);

    module_contract.remove_task(active_task)?;

    let proxy_cw20_balance3: cw20::BalanceResponse = mock.query(
        &Cw20QueryMsg::Balance {
            address: account.proxy.addr_str()?,
        },
        &cw20_addr,
    )?;

    assert!(proxy_cw20_balance3.balance > proxy_cw20_balance2.balance);
    Ok(())
}

#[test]
fn purge() -> anyhow::Result<()> {
    // Set up the environment and contract
    let TestingSetup {
        module_contract,
        account,
        mock,
        ..
    } = setup()?;

    // Task without any cw20s
    let task = TaskRequest {
        interval: croncat_sdk_tasks::types::Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: BankMsg::Send {
                to_address: mock.addr_make("alice").to_string(),
                amount: coins(420, DENOM),
            }
            .into(),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20: None,
    };
    let task_tag = "test_tag".to_owned();
    let assets = AssetListUnchecked::from(AssetList::from(coins(45_000, DENOM)));
    module_contract.create_task(assets, Box::new(task), task_tag)?;

    let active_tasks_by_creator_response: ActiveTasksByCreatorResponse =
        module_contract.active_tasks_by_creator(account.manager.addr_str()?, None, None, None)?;
    let tasks = active_tasks_by_creator_response.unchecked();
    assert_eq!(tasks.len(), 1);

    module_contract.purge(tasks)?;

    let active_tasks_by_creator_response: ActiveTasksByCreatorResponse =
        module_contract.active_tasks_by_creator(account.manager.addr_str()?, None, None, None)?;
    let tasks = active_tasks_by_creator_response.unchecked();
    assert_eq!(tasks.len(), 0);
    Ok(())
}
