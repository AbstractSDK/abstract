mod common;
use abstract_app::gen_app_mock;
use abstract_core::objects::account::TEST_ACCOUNT_ID;
use abstract_core::PROXY;
use abstract_interface::*;

use common::{create_default_account, AResult, TEST_COIN};
use cosmwasm_std::{coin, Addr, Coin, CosmosMsg};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use speculoos::prelude::*;

const APP_ID: &str = "tester:app";
const APP_VERSION: &str = "1.0.0";
gen_app_mock!(MockApp, APP_ID, APP_VERSION, &[]);

#[test]
fn execute_on_proxy_through_manager() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
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

    Ok(())
}

#[test]
fn account_install_app() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    app.deploy(APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    let app_addr = account.install_app(app, &MockInitMsg, None)?;
    let module_addr = account.manager.module_info(APP_ID)?.unwrap().address;

    assert_that!(app_addr).is_equal_to(module_addr);

    let contract_info = chain
        .app
        .borrow()
        .wrap()
        .query_wasm_contract_info(app_addr)?;
    println!("contract_info {contract_info:?}");
    Ok(())
}
