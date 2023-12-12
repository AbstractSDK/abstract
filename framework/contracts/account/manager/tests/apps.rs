mod common;
use abstract_app::mock::MockError;
use abstract_app::{gen_app_mock, mock};
use abstract_core::manager::ModuleInstallConfig;
use abstract_core::objects::account::TEST_ACCOUNT_ID;
use abstract_core::objects::module::ModuleInfo;
use abstract_core::objects::AccountId;
use abstract_core::PROXY;
use abstract_interface::*;

use abstract_testing::prelude::*;
use common::{create_default_account, AResult, TEST_COIN};
use cosmwasm_std::{coin, to_json_binary, Addr, Coin, CosmosMsg};
use cw_controllers::{AdminError, AdminResponse};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use speculoos::prelude::*;

const APP_ID: &str = "tester:app";
const APP_VERSION: &str = "1.0.0";
gen_app_mock!(MockApp, APP_ID, APP_VERSION, &[]);

#[test]
fn execute_on_proxy_through_manager() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    // mint coins to proxy address
    chain.set_balance(
        &account.proxy.address()?,
        vec![Coin::new(100_000, TEST_COIN)],
    )?;
    // mint other coins to owner
    chain.set_balance(&sender, vec![Coin::new(100, "other_coin")])?;

    // burn coins from proxy
    let proxy_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000, TEST_COIN)]);

    let burn_amount: Vec<Coin> = vec![Coin::new(10_000, TEST_COIN)];
    let forwarded_coin: Coin = coin(100, "other_coin");

    account.manager.exec_on_module(
        cosmwasm_std::to_json_binary(&abstract_core::proxy::ExecuteMsg::ModuleAction {
            msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
                amount: burn_amount,
            })],
        })?,
        PROXY.to_string(),
        &[forwarded_coin.clone()],
    )?;

    let proxy_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.proxy.address()?)?;
    assert_that!(proxy_balance)
        .is_equal_to(vec![forwarded_coin, Coin::new(100_000 - 10_000, TEST_COIN)]);

    take_storage_snapshot!(chain, "execute_on_proxy_through_manager");

    Ok(())
}

#[test]
fn account_install_app() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    let app_addr = account.install_app(&app, &MockInitMsg {}, None)?;
    let module_addr = account.manager.module_info(APP_ID)?.unwrap().address;
    assert_that!(app_addr).is_equal_to(module_addr);
    take_storage_snapshot!(chain, "account_install_app");
    Ok(())
}

#[test]
fn account_app_ownership() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    account.install_app(&app, &MockInitMsg {}, None)?;

    let admin_res: AdminResponse =
        app.query(&mock::QueryMsg::Base(app::BaseQueryMsg::BaseAdmin {}))?;
    assert_eq!(admin_res.admin.unwrap(), account.manager.address()?);

    // Can call either by account owner or manager
    app.call_as(&sender).execute(
        &mock::ExecuteMsg::Module(MockExecMsg::DoSomethingAdmin {}),
        None,
    )?;
    app.call_as(&account.manager.address()?).execute(
        &mock::ExecuteMsg::Module(MockExecMsg::DoSomethingAdmin {}),
        None,
    )?;

    // Not admin or manager
    let err: MockError = app
        .call_as(&Addr::unchecked("who"))
        .execute(
            &mock::ExecuteMsg::Module(MockExecMsg::DoSomethingAdmin {}),
            None,
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, MockError::Admin(AdminError::NotAdmin {}));
    Ok(())
}

#[test]
fn subaccount_app_ownership() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;

    account.manager.create_sub_account(
        vec![ModuleInstallConfig::new(
            ModuleInfo::from_id_latest(APP_ID).unwrap(),
            Some(to_json_binary(&MockInitMsg {}).unwrap()),
        )],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;

    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));
    let module = sub_account.manager.module_info(APP_ID)?.unwrap();
    app.set_address(&module.address);
    let admin_res: AdminResponse =
        app.query(&mock::QueryMsg::Base(app::BaseQueryMsg::BaseAdmin {}))?;
    assert_eq!(admin_res.admin.unwrap(), sub_account.manager.address()?);
    app.call_as(&sender).execute(
        &mock::ExecuteMsg::Module(MockExecMsg::DoSomethingAdmin {}),
        None,
    )?;
    Ok(())
}
