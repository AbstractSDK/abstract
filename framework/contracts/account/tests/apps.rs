use core::panic;

use abstract_account::error::AccountError;
use abstract_app::{gen_app_mock, mock, mock::MockError};
use abstract_integration_tests::{create_default_account, AResult};
use abstract_interface::*;
use abstract_std::{
    account::ModuleInstallConfig,
    objects::{
        gov_type::TopLevelOwnerResponse,
        module::{ModuleInfo, ModuleStatus, ModuleVersion},
        AccountId,
    },
    registry::ModuleFilter,
};
use abstract_testing::prelude::*;
use cosmwasm_std::{coin, CosmosMsg};
use cw_controllers::{AdminError, AdminResponse};
use cw_orch::prelude::*;

const APP_ID: &str = "tester:app";
const APP_VERSION: &str = "1.0.0";
gen_app_mock!(MockApp, APP_ID, APP_VERSION, &[]);

#[test]
fn execute_on_account() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let account = create_default_account(&sender, &deployment)?;

    // mint coins to account address
    chain.set_balance(&account.address()?, vec![Coin::new(100_000u128, TTOKEN)])?;
    // mint other coins to owner
    chain.set_balance(&sender, vec![Coin::new(100u128, "other_coin")])?;

    // burn coins from account
    let account_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.address()?)?;
    assert_eq!(account_balance, vec![Coin::new(100_000u128, TTOKEN)]);

    let burn_amount: Vec<Coin> = vec![Coin::new(10_000u128, TTOKEN)];
    let forwarded_coin: Coin = coin(100, "other_coin");

    account.execute(
        &abstract_std::account::ExecuteMsg::Execute {
            msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
                amount: burn_amount,
            })],
        },
        &[forwarded_coin.clone()],
    )?;

    let account_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.address()?)?;
    assert_eq!(
        account_balance,
        vec![
            forwarded_coin,
            Coin::new((100_000 - 10_000) as u128, TTOKEN),
        ]
    );

    take_storage_snapshot!(chain, "execute_on_account");

    Ok(())
}

#[test]
fn account_install_app() -> AResult {
    let chain = MockBech32::new("mock");
    Abstract::deploy_on(chain.clone(), ())?;
    abstract_integration_tests::account::account_install_app(chain.clone())?;
    take_storage_snapshot!(chain, "account_install_app");
    Ok(())
}

#[test]
fn account_install_app_without_init_msg() -> AResult {
    let chain = MockBech32::new("mock");
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let account = crate::create_default_account(&chain.sender_addr(), &deployment)?;

    deployment
        .registry
        .claim_namespace(account.id()?, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    MockApp::deploy(&app, APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    account
        .install_module::<Empty>(APP_ID, None, &[])
        .unwrap_err();
    Ok(())
}

#[test]
fn account_app_ownership() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let account = create_default_account(&sender, &deployment)?;

    deployment
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    account.install_app(&app, &MockInitMsg {}, &[])?;

    let admin_res: AdminResponse =
        app.query(&mock::QueryMsg::Base(app::BaseQueryMsg::BaseAdmin {}))?;
    assert_eq!(admin_res.admin.unwrap(), account.addr_str()?);

    // Can call either by account owner or account
    app.call_as(&sender).execute(
        &mock::ExecuteMsg::Module(MockExecMsg::DoSomethingAdmin {}),
        &[],
    )?;
    account.call_as(&sender).admin_execute(
        app.address()?,
        to_json_binary(&mock::ExecuteMsg::Module(MockExecMsg::DoSomethingAdmin {}))?,
        &[],
    )?;

    // Account cannot call by itself without the CALLING_TO variable set
    app.call_as(&account.address()?)
        .execute(
            &mock::ExecuteMsg::Module(MockExecMsg::DoSomethingAdmin {}),
            &[],
        )
        .unwrap_err();

    // Not admin or account
    let err: MockError = app
        .call_as(&Addr::unchecked("who"))
        .execute(
            &mock::ExecuteMsg::Module(MockExecMsg::DoSomethingAdmin {}),
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, MockError::Admin(AdminError::NotAdmin {}));
    Ok(())
}

#[test]
fn subaccount_app_ownership() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let account = create_default_account(&sender, &deployment)?;

    deployment
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;

    let next_id = deployment.registry.config()?.local_account_sequence;

    account.create_sub_account(
        vec![ModuleInstallConfig::new(
            ModuleInfo::from_id_latest(APP_ID).unwrap(),
            Some(to_json_binary(&MockInitMsg {}).unwrap()),
        )],
        None,
        None,
        None,
        Some("My subaccount".to_string()),
        None,
        &[],
    )?;

    let sub_account = AccountI::load_from(&deployment, AccountId::local(next_id))?;
    let module = sub_account.module_info(APP_ID)?.unwrap();
    app.set_address(&module.address);

    // Check query gives us right Top Level Owner
    let top_level_owner_res: TopLevelOwnerResponse =
        app.query(&mock::QueryMsg::Base(app::BaseQueryMsg::TopLevelOwner {}))?;
    assert_eq!(top_level_owner_res.address, sender);

    let admin_res: AdminResponse =
        app.query(&mock::QueryMsg::Base(app::BaseQueryMsg::BaseAdmin {}))?;
    assert_eq!(admin_res.admin.unwrap(), sub_account.addr_str()?);
    app.call_as(&sender).execute(
        &mock::ExecuteMsg::Module(MockExecMsg::DoSomethingAdmin {}),
        &[],
    )?;
    Ok(())
}

#[test]
fn cant_reinstall_app_after_uninstall() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let account = create_default_account(&sender, &deployment)?;

    deployment
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    account.install_app(&app, &MockInitMsg {}, &[])?;

    // Reinstall
    account.uninstall_module(APP_ID.to_owned())?;

    let Err(AbstractInterfaceError::Orch(err)) = account.install_app(&app, &MockInitMsg {}, &[])
    else {
        panic!("Expected error");
    };
    let account_err: AccountError = err.downcast().unwrap();
    assert_eq!(account_err, AccountError::ProhibitedReinstall {});
    Ok(())
}

#[test]
fn deploy_strategy_uploaded() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let _account = create_default_account(&sender, &deployment)?;

    deployment
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;
    deployment.registry.update_config(None, Some(true))?;

    let app = MockApp::new_test(chain.clone());
    app.upload()?;

    // Deploy try
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    let module_list = deployment.registry.module_list(
        Some(ModuleFilter {
            status: Some(ModuleStatus::Pending),
            ..Default::default()
        }),
        None,
        None,
    )?;
    assert!(module_list.modules[0].module.info.name == "app");

    // Clean module
    deployment.registry.approve_or_reject_modules(
        vec![],
        vec![ModuleInfo::from_id(
            APP_ID,
            ModuleVersion::Version(APP_VERSION.to_owned()),
        )?],
    )?;

    // Deploy Error
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Error)?;
    let module_list = deployment.registry.module_list(
        Some(ModuleFilter {
            status: Some(ModuleStatus::Pending),
            ..Default::default()
        }),
        None,
        None,
    )?;
    assert!(module_list.modules[0].module.info.name == "app");

    // Clean module
    deployment.registry.approve_or_reject_modules(
        vec![],
        vec![ModuleInfo::from_id(
            APP_ID,
            ModuleVersion::Version(APP_VERSION.to_owned()),
        )?],
    )?;

    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Force)?;
    let module_list = deployment.registry.module_list(
        Some(ModuleFilter {
            status: Some(ModuleStatus::Pending),
            ..Default::default()
        }),
        None,
        None,
    )?;
    assert!(module_list.modules[0].module.info.name == "app");

    Ok(())
}

#[test]
fn deploy_strategy_deployed() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let _account = create_default_account(sender, &deployment)?;

    deployment
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;
    deployment.registry.update_config(None, Some(true))?;

    let app = MockApp::new_test(chain.clone());

    // deploy (not approved)
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;

    // Deploy try
    let try_res = app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try);
    assert!(try_res.is_ok());

    // Deploy Error
    let error_res = app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Error);
    assert!(error_res.is_err());

    // Deploy Force
    let force_res = app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Force);
    // App not updatable
    assert!(force_res.is_err());
    Ok(())
}
