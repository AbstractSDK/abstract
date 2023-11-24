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

    take_storage_snapshot!(chain, "execute_on_proxy_through_manager");

    Ok(())
}

#[test]
fn account_install_app() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    abstract_integration_tests::manager::account_install_app(chain.clone(), sender)?;
    take_storage_snapshot!(chain, "account_install_app");
    Ok(())
}
