mod common;

use abstract_interface::*;
use common::*;
use cosmwasm_std::{wasm_execute, Addr};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
// use cw_multi_test::StakingInfo;

#[test]
fn creating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account
        .manager
        .create_sub_account("My subaccount".to_string(), None, None, None, None)?;
    Ok(())
}

#[test]
fn updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account
        .manager
        .create_sub_account("My subaccount".to_string(), None, None, None, None)?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let account_contracts = get_account_contracts(&deployment.version_control, Some(2));
    let new_desc = "new desc";
    account_contracts
        .0
        .update_info(Some(new_desc.to_string()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        account_contracts.0.info()?.info.description
    );

    Ok(())
}

#[test]
fn proxy_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let proxy_address = account.proxy.address()?;
    account
        .manager
        .create_sub_account("My subaccount".to_string(), None, None, None, None)?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let (sub_manager, _sub_proxy) = get_account_contracts(&deployment.version_control, Some(2));
    let new_desc = "new desc";

    // We call as the proxy, it should also be possible
    sub_manager
        .call_as(&proxy_address)
        .update_info(Some(new_desc.to_owned()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        sub_manager.info()?.info.description
    );

    Ok(())
}

#[test]
fn recursive_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account
        .manager
        .create_sub_account("My subaccount".to_string(), None, None, None, None)?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let account_contracts = get_account_contracts(&deployment.version_control, Some(2));

    // We call as the manager, it should also be possible
    account_contracts.0.create_sub_account(
        "My subsubaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    let account_contracts = get_account_contracts(&deployment.version_control, Some(3));
    let new_desc = "new desc";

    account_contracts
        .0
        .update_info(Some(new_desc.to_string()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        account_contracts.0.info()?.info.description
    );

    Ok(())
}

#[test]
fn installed_app_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account
        .manager
        .create_sub_account("My subaccount".to_string(), None, None, None, None)?;
    let first_proxy_addr = account.proxy.address()?;

    let mock_app = Addr::unchecked("mock_app");
    account
        .proxy
        .call_as(&account.manager.address()?)
        .add_module(mock_app.to_string())?;

    let (sub_manager, _sub_proxy) = get_account_contracts(&deployment.version_control, Some(2));
    let new_desc = "new desc";

    // recover address on first proxy
    account.proxy.set_address(&first_proxy_addr);
    // adding mock_app to whitelist on proxy

    // We call as installed app of the owner-proxy, it should also be possible
    account
        .proxy
        .call_as(&mock_app)
        .module_action(vec![wasm_execute(
            sub_manager.addr_str()?,
            &abstract_core::manager::ExecuteMsg::UpdateInfo {
                name: None,
                description: Some(new_desc.to_owned()),
                link: None,
            },
            vec![],
        )?
        .into()])?;

    assert_eq!(
        Some(new_desc.to_string()),
        sub_manager.info()?.info.description
    );

    Ok(())
}
