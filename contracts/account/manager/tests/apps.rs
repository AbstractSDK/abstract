mod common;
use abstract_boot::*;
use abstract_core::PROXY;
use abstract_testing::prelude::TEST_VERSION;
use boot_core::{instantiate_default_mock_env, ContractInstance, Deploy};
use common::{create_default_account, AResult, TEST_COIN};
use cosmwasm_std::{Addr, Coin, CosmosMsg};
use speculoos::prelude::*;

#[test]
fn execute_on_proxy_through_manager() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;

    // mint coins to proxy address
    chain.set_balance(
        &account.proxy.address()?,
        vec![Coin::new(100_000, TEST_COIN)],
    )?;

    // burn coins from proxy
    let proxy_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000, TEST_COIN)]);

    let burn_amount: Vec<Coin> = vec![Coin::new(10_000, TEST_COIN)];

    account.manager.exec_on_module(
        cosmwasm_std::to_binary(&abstract_core::proxy::ExecuteMsg::ModuleAction {
            msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
                amount: burn_amount,
            })],
        })?,
        PROXY.to_string(),
    )?;

    let proxy_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000 - 10_000, TEST_COIN)]);

    Ok(())
}
