use abstract_integration_tests::{create_default_account, AResult};
use abstract_interface::*;
use abstract_manager::error::ManagerError;
use abstract_std::{
    manager::{self, SubAccountIdsResponse},
    objects::{
        gov_type::{GovAction, GovernanceDetails},
        ownership, AccountId,
    },
    proxy, ACCOUNT,
};
use cosmwasm_std::{to_json_binary, wasm_execute, WasmMsg};
use cw_orch::prelude::*;

#[test]
fn creating_on_subaccount_should_succeed() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;
    let sub_accounts = account.account.sub_account_ids(None, None)?;
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
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
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
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let proxy_address = account.proxy.address()?;
    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
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
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
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
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;
    let first_proxy_addr = account.proxy.address()?;

    let mock_app = chain.addr_make("mock_app");
    account
        .proxy
        .call_as(&account.account.address()?)
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
            &abstract_std::manager::ExecuteMsg::UpdateInfo {
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
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let new_owner = chain.addr_make("new_owner");
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    // Store manager address, it will be used for querying
    let manager_addr = account.account.address()?;

    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;
    let sub_accounts = account.account.sub_account_ids(None, None)?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            // only one sub-account and it should be account_id 2
            sub_accounts: vec![2]
        }
    );

    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));
    sub_account
        .account
        .update_ownership(GovAction::TransferOwnership {
            new_owner: GovernanceDetails::Monarchy {
                monarch: new_owner.to_string(),
            },
            expiry: None,
        })?;

    // Make sure it's not updated until claimed
    let sub_accounts: SubAccountIdsResponse = chain.query(
        &abstract_std::manager::QueryMsg::SubAccountIds {
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
    sub_account.account.call_as(&new_owner).execute(
        &abstract_std::manager::ExecuteMsg::UpdateOwnership(ownership::GovAction::AcceptOwnership),
        None,
    )?;
    let account = AbstractAccount::new(&deployment, AccountId::local(1));

    // After claim it's updated
    let sub_accounts = account.account.sub_account_ids(None, None)?;
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
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    Abstract::deploy_on(chain.clone(), sender.to_string())?;
    abstract_integration_tests::manager::account_move_ownership_to_sub_account(chain)?;
    Ok(())
}

#[test]
fn sub_account_move_ownership_to_sub_account() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;
    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));
    let sub_manager_addr = sub_account.account.address()?;
    let sub_proxy_addr = sub_account.proxy.address()?;

    let new_account = create_default_account(&deployment.account_factory)?;
    new_account.account.create_sub_account(
        vec![],
        "My second subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;

    // sub-accounts state updated
    let sub_ids = new_account.account.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![4]);

    let new_account_sub_account = AbstractAccount::new(&deployment, AccountId::local(4));
    let new_governance = GovernanceDetails::SubAccount {
        manager: sub_manager_addr.to_string(),
        proxy: sub_proxy_addr.to_string(),
    };
    new_account_sub_account
        .account
        .update_ownership(GovAction::TransferOwnership {
            new_owner: new_governance.clone(),
            expiry: None,
        })?;
    let new_account_sub_account_manager = new_account_sub_account.account.address()?;

    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));
    let mock_module = chain.addr_make("mock_module");
    sub_account
        .proxy
        .call_as(&sub_manager_addr)
        .add_modules(vec![mock_module.to_string()])?;
    sub_account
        .proxy
        .call_as(&mock_module)
        .module_action(vec![wasm_execute(
            new_account_sub_account_manager,
            &abstract_std::manager::ExecuteMsg::UpdateOwnership(
                ownership::GovAction::AcceptOwnership,
            ),
            vec![],
        )?
        .into()])?;

    // sub-accounts state updated
    let sub_ids = sub_account.account.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![4]);
    let new_account = AbstractAccount::new(&deployment, AccountId::local(3));
    // removed from the previous owner as well
    let sub_ids = new_account.account.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, Vec::<u32>::new());

    let new_account_sub_account = AbstractAccount::new(&deployment, AccountId::local(4));
    let info = new_account_sub_account.account.ownership()?;
    assert_eq!(new_governance, info.owner);
    take_storage_snapshot!(chain, "sub_account_move_ownership_to_sub_account");

    Ok(())
}

#[test]
fn account_move_ownership_to_falsy_sub_account() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let proxy_addr = account.proxy.address()?;

    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;
    let sub_account = AbstractAccount::new(&deployment, AccountId::local(2));
    let sub_manager_addr = sub_account.account.address()?;

    let new_account = create_default_account(&deployment.account_factory)?;
    // proxy and manager of different accounts
    let new_governance = GovernanceDetails::SubAccount {
        manager: sub_manager_addr.to_string(),
        proxy: proxy_addr.to_string(),
    };
    let err = new_account
        .account
        .update_ownership(GovAction::TransferOwnership {
            new_owner: new_governance.clone(),
            expiry: None,
        })
        .unwrap_err();
    let err = err.root().to_string();
    assert!(err.contains("manager and proxy has different account ids"));
    take_storage_snapshot!(chain, "account_move_ownership_to_falsy_sub_account");
    Ok(())
}

#[test]
fn account_updated_to_subaccount() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    // Creating account1
    let account = create_default_account(&deployment.account_factory)?;
    let proxy1_addr = account.proxy.address()?;
    let manager1_addr = account.account.address()?;

    // Creating account2
    let account = create_default_account(&deployment.account_factory)?;
    let manager2_addr = account.account.address()?;

    // Setting account1 as pending owner of account2
    account
        .account
        .update_ownership(GovAction::TransferOwnership {
            new_owner: GovernanceDetails::SubAccount {
                manager: manager1_addr.to_string(),
                proxy: proxy1_addr.to_string(),
            },
            expiry: None,
        })?;
    account.account.set_address(&manager1_addr);
    account.proxy.set_address(&proxy1_addr);

    // account1 accepting account2 as a sub-account
    let accept_msg =
        abstract_std::manager::ExecuteMsg::UpdateOwnership(ownership::GovAction::AcceptOwnership);
    account.account.exec_on_module(
        to_json_binary(&abstract_std::proxy::ExecuteMsg::ModuleAction {
            msgs: vec![wasm_execute(manager2_addr, &accept_msg, vec![])?.into()],
        })?,
        ACCOUNT.to_owned(),
        &[],
    )?;

    // Check manager knows about his new sub-account
    let ids = account.account.sub_account_ids(None, None)?;
    assert_eq!(ids.sub_accounts.len(), 1);
    Ok(())
}

#[test]
fn account_updated_to_subaccount_recursive() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    // Creating account1
    let account = create_default_account(&deployment.account_factory)?;
    let proxy1_addr = account.proxy.address()?;
    let manager1_addr = account.account.address()?;

    // Creating account2
    let account = create_default_account(&deployment.account_factory)?;

    // Setting account1 as pending owner of account2
    account
        .account
        .update_ownership(GovAction::TransferOwnership {
            new_owner: GovernanceDetails::SubAccount {
                manager: manager1_addr.to_string(),
                proxy: proxy1_addr.to_string(),
            },
            expiry: None,
        })?;
    // accepting ownership by sender instead of the manager
    account
        .account
        .update_ownership(ownership::GovAction::AcceptOwnership)?;

    // Check manager knows about his new sub-account
    account.account.set_address(&manager1_addr);
    let ids = account.account.sub_account_ids(None, None)?;
    assert_eq!(ids.sub_accounts.len(), 1);
    Ok(())
}

#[test]
fn top_level_owner() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let account = create_default_account(&deployment.account_factory)?;
    // Creating sub account
    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;
    let response = account.account.sub_account_ids(None, None)?;
    let sub_account = AbstractAccount::new(&deployment, AccountId::local(response.sub_accounts[0]));

    let top_level_owner = sub_account.account.top_level_owner()?;
    assert_eq!(top_level_owner.address, sender);
    Ok(())
}

#[test]
fn cant_renounce_with_sub_accounts() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let account = create_default_account(&deployment.account_factory)?;
    // Creating sub account
    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;

    let err: ManagerError = account
        .account
        .update_ownership(ownership::GovAction::RenounceOwnership)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, ManagerError::RenounceWithSubAccount {});
    Ok(())
}

#[test]
fn can_renounce_sub_accounts() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let account = create_default_account(&deployment.account_factory)?;
    // Creating sub account
    account.account.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;

    let sub_account_id = account.account.sub_account_ids(None, None)?.sub_accounts[0];

    let sub_account = AbstractAccount::new(&deployment, AccountId::local(sub_account_id));

    sub_account
        .account
        .update_ownership(ownership::GovAction::RenounceOwnership)?;

    account
        .account
        .update_ownership(ownership::GovAction::RenounceOwnership)?;

    // No owners
    // Renounced governance
    let account_owner = account.account.ownership()?;
    assert_eq!(account_owner.owner, GovernanceDetails::Renounced {});
    let sub_account_owner = sub_account.account.ownership()?;
    assert_eq!(sub_account_owner.owner, GovernanceDetails::Renounced {});

    Ok(())
}

#[test]
fn account_updated_to_subaccount_without_recursion() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    // Creating account1
    let account_1 = create_default_account(&deployment.account_factory)?;

    // Creating account2
    let account_2 = create_default_account(&deployment.account_factory)?;

    // Setting account1 as pending owner of account2
    account_2
        .account
        .update_ownership(GovAction::TransferOwnership {
            new_owner: GovernanceDetails::SubAccount {
                manager: account_1.account.addr_str()?,
                proxy: account_1.proxy.addr_str()?,
            },
            expiry: None,
        })?;

    // accepting ownership by sender instead of the manager
    account_1.account.execute_on_module(
        ACCOUNT,
        proxy::ExecuteMsg::ModuleAction {
            msgs: vec![WasmMsg::Execute {
                contract_addr: account_2.account.addr_str()?,
                msg: to_json_binary(&manager::ExecuteMsg::UpdateOwnership(
                    GovAction::AcceptOwnership,
                ))?,
                funds: vec![],
            }
            .into()],
        },
    )?;

    // Check manager knows about his new sub-account
    let ids = account_1.account.sub_account_ids(None, None)?;
    assert_eq!(ids.sub_accounts.len(), 1);
    Ok(())
}

#[test]
fn sub_account_to_regular_account_without_recursion() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    // Creating account1
    let account = create_default_account(&deployment.account_factory)?;
    let sub_account = account.create_sub_account(
        AccountDetails {
            name: "sub_account".to_owned(),
            ..Default::default()
        },
        None,
    )?;

    account.account.execute_on_module(
        ACCOUNT,
        proxy::ExecuteMsg::ModuleAction {
            msgs: vec![WasmMsg::Execute {
                contract_addr: sub_account.account.addr_str()?,
                msg: to_json_binary(&manager::ExecuteMsg::UpdateOwnership(
                    GovAction::TransferOwnership {
                        new_owner: GovernanceDetails::Monarchy {
                            monarch: chain.sender_addr().to_string(),
                        },
                        expiry: None,
                    },
                ))?,
                funds: vec![],
            }
            .into()],
        },
    )?;

    sub_account
        .account
        .update_ownership(GovAction::AcceptOwnership)?;
    let ownership = sub_account.account.ownership()?;
    assert_eq!(
        ownership.owner,
        GovernanceDetails::Monarchy {
            monarch: chain.sender_addr().to_string()
        }
    );
    Ok(())
}
