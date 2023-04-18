mod common;
use abstract_boot::*;
use abstract_core::{manager::ManagerModuleInfo, PROXY};
use abstract_manager::contract::CONTRACT_VERSION;
use abstract_testing::prelude::TEST_VERSION;
use boot_core::{instantiate_default_mock_env, ContractInstance, Deploy};
use common::{create_default_account, AResult, TEST_COIN};
use cosmwasm_std::{Addr, Coin, CosmosMsg};
use speculoos::prelude::*;

#[test]
fn instantiate() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain, TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;

    let modules = account.manager.module_infos(None, None)?.module_infos;

    // assert proxy module
    assert_that!(&modules).has_length(1);
    assert_that(&modules[0]).is_equal_to(&ManagerModuleInfo {
        address: account.proxy.address()?,
        id: PROXY.to_string(),
        version: cw2::ContractVersion {
            contract: PROXY.into(),
            version: CONTRACT_VERSION.into(),
        },
    });

    // assert manager config
    assert_that!(account.manager.config()?).is_equal_to(abstract_core::manager::ConfigResponse {
        owner: sender.to_string(),
        version_control_address: deployment.version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        account_id: 0u32.into(),
        is_suspended: false,
    });
    Ok(())
}

#[test]
fn exec_through_manager() -> AResult {
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

/// This basically just checks that the proxy is able to be migrated .... but the actual version cannot change... unless we mock the responses from the version queries
#[test]
fn migrate_proxy() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain, TEST_VERSION.parse().unwrap())?;
    let account = create_default_account(&deployment.account_factory)?;

    let new_version = "1.0.1".parse().unwrap();
    deployment
        .version_control
        .register_account_mods(vec![account.proxy.as_instance()], &new_version)?;

    account
        .manager
        .upgrade_module(PROXY, &abstract_core::proxy::MigrateMsg {})?;

    let module = account.manager.module_info(PROXY)?;

    assert_that!(module)
        .is_some()
        .map(|m| &m.version)
        .is_equal_to(cw2::ContractVersion {
            contract: PROXY.into(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        });

    Ok(())
}
