use abstract_core::{
    manager::SubAccountIdsResponse,
    objects::{gov_type::GovernanceDetails, AccountId},
    PROXY,
};
use abstract_integration_tests::{create_default_account, AResult};
use abstract_interface::*;
use abstract_manager::error::ManagerError;
use abstract_testing::OWNER;
use cosmwasm_std::{to_json_binary, wasm_execute, Addr};
use cw_orch::{contract::Deploy, prelude::*};

#[test]
fn creating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;
    let sub_accounts = account.manager.sub_account_ids(None, None)?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            // only one sub-account and it should be account_id 2
            sub_accounts: vec![2]
        }
    );
    take_storage_snapshot!(chain, "creating_on_subaccount_should_succeed");
    Ok(())
}

#[test]
fn updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let account_contracts = get_account_contracts(&deployment.version_control, AccountId::local(2));
    let new_desc = "new desc";
    account_contracts
        .0
        .update_info(Some(new_desc.to_string()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        account_contracts.0.info()?.info.description
    );
    take_storage_snapshot!(chain, "updating_on_subaccount_should_succeed");
    Ok(())
}

#[test]
fn proxy_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let proxy_address = account.proxy.address()?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let (sub_manager, _) = get_account_contracts(&deployment.version_control, AccountId::local(2));
    let new_desc = "new desc";

    // We call as the proxy, it should also be possible
    sub_manager
        .call_as(&proxy_address)
        .update_info(Some(new_desc.to_owned()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        sub_manager.info()?.info.description
    );

    take_storage_snapshot!(chain, "proxy_updating_on_subaccount_should_succeed");
    Ok(())
}

#[test]
fn recursive_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;

    // Subaccount should have id 2 in this test, we try to update the config of this module
    let account_contracts = get_account_contracts(&deployment.version_control, AccountId::local(2));

    // We call as the manager, it should also be possible
    account_contracts.0.create_sub_account(
        vec![],
        "My subsubaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;
    let account_contracts = get_account_contracts(&deployment.version_control, AccountId::local(3));
    let new_desc = "new desc";

    account_contracts
        .0
        .call_as(&sender)
        .update_info(Some(new_desc.to_string()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        account_contracts.0.info()?.info.description
    );

    take_storage_snapshot!(chain, "recursive_updating_on_subaccount_should_succeed");
    Ok(())
}

#[test]
fn installed_app_updating_on_subaccount_should_succeed() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;
    let first_proxy_addr = account.proxy.address()?;

    let mock_app = Addr::unchecked("mock_app");
    account
        .proxy
        .call_as(&account.manager.address()?)
        .add_modules(vec![mock_app.to_string()])?;

    let (sub_manager, _sub_proxy) =
        get_account_contracts(&deployment.version_control, AccountId::local(2));
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
    take_storage_snapshot!(chain, "installed_app_updating_on_subaccount_should_succeed");

    Ok(())
}

#[test]
fn sub_account_move_ownership() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let new_owner = Addr::unchecked("new_owner");
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    // Store manager address, it will be used for querying
    let manager_addr = account.manager.address()?;

    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;
    let sub_accounts = account.manager.sub_account_ids(None, None)?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            // only one sub-account and it should be account_id 2
            sub_accounts: vec![2]
        }
    );

    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));
    sub_account
        .manager
        .propose_owner(GovernanceDetails::Monarchy {
            monarch: new_owner.to_string(),
        })?;

    // Make sure it's not updated until claimed
    let sub_accounts: SubAccountIdsResponse = chain.query(
        &abstract_core::manager::QueryMsg::SubAccountIds {
            start_after: None,
            limit: None,
        },
        &manager_addr,
    )?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            sub_accounts: vec![2]
        }
    );

    // Claim ownership
    sub_account.manager.call_as(&new_owner).execute(
        &abstract_core::manager::ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership),
        None,
    )?;
    let account = AbstractAccount::new(&deployment, AccountId::local(1));

    // After claim it's updated
    let sub_accounts = account.manager.sub_account_ids(None, None)?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            sub_accounts: vec![]
        }
    );
    take_storage_snapshot!(chain, "sub_account_move_ownership");

    Ok(())
}

#[test]
fn account_move_ownership_to_sub_account() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    Abstract::deploy_on(chain.clone(), sender.to_string())?;
    abstract_integration_tests::manager::account_move_ownership_to_sub_account(chain)?;
    Ok(())
}

#[test]
fn sub_account_move_ownership_to_sub_account() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;
    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));
    let sub_manager_addr = sub_account.manager.address()?;
    let sub_proxy_addr = sub_account.proxy.address()?;

    let new_account = create_default_account(&deployment.account_factory)?;
    new_account.manager.create_sub_account(
        vec![],
        "My second subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;

    // sub-accounts state updated
    let sub_ids = new_account.manager.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![4]);

    let new_account_sub_account = AbstractAccount::new(&deployment, AccountId::local(4));
    let new_governance = GovernanceDetails::SubAccount {
        manager: sub_manager_addr.to_string(),
        proxy: sub_proxy_addr.to_string(),
    };
    new_account_sub_account
        .manager
        .propose_owner(new_governance.clone())?;
    let new_account_sub_account_manager = new_account_sub_account.manager.address()?;

    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));
    let mock_module = Addr::unchecked("mock_module");
    sub_account
        .proxy
        .call_as(&sub_manager_addr)
        .add_modules(vec![mock_module.to_string()])?;
    sub_account
        .proxy
        .call_as(&mock_module)
        .module_action(vec![wasm_execute(
            new_account_sub_account_manager,
            &abstract_core::manager::ExecuteMsg::UpdateOwnership(
                cw_ownable::Action::AcceptOwnership,
            ),
            vec![],
        )?
        .into()])?;

    // sub-accounts state updated
    let sub_ids = sub_account.manager.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![4]);
    let new_account = AbstractAccount::new(&deployment, AccountId::local(3));
    // removed from the previous owner as well
    let sub_ids = new_account.manager.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, Vec::<u32>::new());

    let new_account_sub_account = AbstractAccount::new(&deployment, AccountId::local(4));
    let info = new_account_sub_account.manager.info()?.info;
    assert_eq!(new_governance, info.governance_details.into());
    take_storage_snapshot!(chain, "sub_account_move_ownership_to_sub_account");

    Ok(())
}

#[test]
fn account_move_ownership_to_falsy_sub_account() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let proxy_addr = account.proxy.address()?;

    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;
    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));
    let sub_manager_addr = sub_account.manager.address()?;

    let new_account = create_default_account(&deployment.account_factory)?;
    // proxy and manager of different accounts
    let new_governance = GovernanceDetails::SubAccount {
        manager: sub_manager_addr.to_string(),
        proxy: proxy_addr.to_string(),
    };
    let err = new_account
        .manager
        .propose_owner(new_governance)
        .unwrap_err();
    let err = err.root().to_string();
    assert!(err.contains("manager and proxy has different account ids"));
    take_storage_snapshot!(chain, "account_move_ownership_to_falsy_sub_account");
    Ok(())
}

#[test]
fn account_updated_to_subaccount() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    // Creating account1
    let account = create_default_account(&deployment.account_factory)?;
    let proxy1_addr = account.proxy.address()?;
    let manager1_addr = account.manager.address()?;

    // Creating account2
    let account = create_default_account(&deployment.account_factory)?;
    let manager2_addr = account.manager.address()?;

    // Setting account1 as pending owner of account2
    account
        .manager
        .propose_owner(GovernanceDetails::SubAccount {
            manager: manager1_addr.to_string(),
            proxy: proxy1_addr.to_string(),
        })?;
    account.manager.set_address(&manager1_addr);
    account.proxy.set_address(&proxy1_addr);

    // account1 accepting account2 as a sub-account
    let accept_msg =
        abstract_core::manager::ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
    account.manager.exec_on_module(
        to_json_binary(&abstract_core::proxy::ExecuteMsg::ModuleAction {
            msgs: vec![wasm_execute(manager2_addr, &accept_msg, vec![])?.into()],
        })?,
        PROXY.to_owned(),
        &[],
    )?;

    // Check manager knows about his new sub-account
    let ids = account.manager.sub_account_ids(None, None)?;
    assert_eq!(ids.sub_accounts.len(), 1);
    Ok(())
}

#[test]
fn account_updated_to_subaccount_recursive() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    // Creating account1
    let account = create_default_account(&deployment.account_factory)?;
    let proxy1_addr = account.proxy.address()?;
    let manager1_addr = account.manager.address()?;

    // Creating account2
    let account = create_default_account(&deployment.account_factory)?;

    // Setting account1 as pending owner of account2
    account
        .manager
        .propose_owner(GovernanceDetails::SubAccount {
            manager: manager1_addr.to_string(),
            proxy: proxy1_addr.to_string(),
        })?;
    // accepting ownership by sender instead of the manager
    account
        .manager
        .update_ownership(cw_ownable::Action::AcceptOwnership)?;

    // Check manager knows about his new sub-account
    account.manager.set_address(&manager1_addr);
    let ids = account.manager.sub_account_ids(None, None)?;
    assert_eq!(ids.sub_accounts.len(), 1);
    Ok(())
}

#[test]
fn top_level_owner() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let account = create_default_account(&deployment.account_factory)?;
    // Creating sub account
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;
    let response = account.manager.sub_account_ids(None, None)?;
    let sub_account = AbstractAccount::new(&deployment, AccountId::local(response.sub_accounts[0]));

    let top_level_owner = sub_account.manager.top_level_owner()?;
    assert_eq!(top_level_owner.address, sender);
    Ok(())
}

#[test]
fn cant_renounce_with_sub_accounts() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let account = create_default_account(&deployment.account_factory)?;
    // Creating sub account
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;

    let err: ManagerError = account
        .manager
        .update_ownership(cw_ownable::Action::RenounceOwnership)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ManagerError::RenounceWithSubAccount {});
    Ok(())
}

#[test]
fn can_renounce_sub_accounts() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let account = create_default_account(&deployment.account_factory)?;
    // Creating sub account
    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        None,
        &[],
    )?;

    let sub_account_id = account.manager.sub_account_ids(None, None)?.sub_accounts[0];

    let sub_account = AbstractAccount::new(&deployment, AccountId::local(sub_account_id));

    sub_account
        .manager
        .update_ownership(cw_ownable::Action::RenounceOwnership)?;

    account
        .manager
        .update_ownership(cw_ownable::Action::RenounceOwnership)?;

    // No owners
    let account_owner = account.manager.ownership()?;
    assert!(account_owner.owner.is_none());
    let sub_account_owner = sub_account.manager.ownership()?;
    assert!(sub_account_owner.owner.is_none());

    // Renounced governance
    let account_info = account.manager.info()?;
    assert_eq!(
        account_info.info.governance_details,
        GovernanceDetails::Renounced {}
    );
    let sub_account_info = sub_account.manager.info()?;
    assert_eq!(
        sub_account_info.info.governance_details,
        GovernanceDetails::Renounced {}
    );
    Ok(())
}
