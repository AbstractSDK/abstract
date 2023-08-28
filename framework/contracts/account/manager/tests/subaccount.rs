mod common;

use abstract_core::objects::AccountId;
use abstract_interface::*;
use common::*;
use cosmwasm_std::Addr;
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
// use cw_multi_test::StakingInfo;

#[test]
fn creating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    Ok(())
}

#[test]
fn updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let account_contracts =
        get_account_contracts(&deployment.version_control, Some(AccountId::local(2)));
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
fn manager_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let manager_address = account.manager.address()?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let account_contracts =
        get_account_contracts(&deployment.version_control, Some(AccountId::local(2)));
    let new_desc = "new desc";

    // We call as the manager, it should also be possible
    account_contracts.0.call_as(&manager_address).update_info(
        Some(new_desc.to_string()),
        None,
        None,
    )?;

    assert_eq!(
        Some(new_desc.to_string()),
        account_contracts.0.info()?.info.description
    );

    Ok(())
}

#[test]
fn recursive_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let account_contracts =
        get_account_contracts(&deployment.version_control, Some(AccountId::local(2)));

    // We call as the manager, it should also be possible
    account_contracts.0.create_sub_account(
        vec![],
        "My subsubaccount".to_string(),
        None,
        None,
        None,
        None,
    )?;
    let account_contracts =
        get_account_contracts(&deployment.version_control, Some(AccountId::local(3)));
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
