use abstract_account::error::AccountError;
use abstract_integration_tests::{create_default_account, AResult};
use abstract_interface::*;
use abstract_std::{
    account::SubAccountIdsResponse,
    objects::{
        gov_type::{GovAction, GovernanceDetails},
        ownership,
    },
};
use cosmwasm_std::{to_json_binary, wasm_execute, WasmMsg};
use cw_orch::prelude::*;

#[test]
fn creating_on_subaccount_should_succeed() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &deployment)?;
    account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            description: None,
            link: None,
            namespace: None,
            install_modules: vec![],
            account_id: None,
        },
        &[],
    )?;
    let sub_accounts = account.sub_account_ids(None, None)?;
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
    let deployment = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &deployment)?;
    account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            description: None,
            link: None,
            namespace: None,
            install_modules: vec![],
            account_id: None,
        },
        &[],
    )?;

    let new_desc = "new desc";
    account.update_info(Some(new_desc.to_string()), None, None)?;

    assert_eq!(Some(new_desc.to_string()), account.info()?.info.description);
    take_storage_snapshot!(chain, "updating_on_subaccount_should_succeed");
    Ok(())
}

#[test]
fn proxy_updating_on_subaccount_should_succeed() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &deployment)?;
    let proxy_address = account.address()?;
    let sub_account = account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;
    let new_desc = "new desc";

    // We call as the proxy, it should also be possible
    sub_account
        .call_as(&proxy_address)
        .update_info(Some(new_desc.to_owned()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        sub_account.info()?.info.description
    );

    take_storage_snapshot!(chain, "proxy_updating_on_subaccount_should_succeed");
    Ok(())
}

#[test]
fn recursive_updating_on_subaccount_should_succeed() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &deployment)?;
    let sub_account = account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;
    // We call as the manager, it should also be possible
    let sub_sub_account = sub_account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;
    let new_desc = "new desc";

    sub_sub_account
        .call_as(&sender)
        .update_info(Some(new_desc.to_string()), None, None)?;

    assert_eq!(
        Some(new_desc.to_string()),
        sub_sub_account.info()?.info.description
    );

    take_storage_snapshot!(chain, "recursive_updating_on_subaccount_should_succeed");
    Ok(())
}

#[test]
fn installed_app_updating_on_subaccount_should_succeed() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &deployment)?;
    let sub_account = account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;

    let mock_app = chain.addr_make("mock_app");
    account.update_whitelist(vec![mock_app.to_string()], Vec::default())?;

    let new_desc = "new desc";
    // adding mock_app to whitelist on proxy

    // We call as installed app of the owner-proxy, it should also be possible
    account.call_as(&mock_app).execute_msgs(
        vec![wasm_execute(
            sub_account.addr_str()?,
            &abstract_std::account::ExecuteMsg::<Empty>::UpdateInfo {
                name: None,
                description: Some(new_desc.to_owned()),
                link: None,
            },
            vec![],
        )?
        .into()],
        &[],
    )?;

    assert_eq!(
        Some(new_desc.to_string()),
        sub_account.info()?.info.description
    );
    take_storage_snapshot!(chain, "installed_app_updating_on_subaccount_should_succeed");

    Ok(())
}

#[test]
fn sub_account_move_ownership() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let new_owner = chain.addr_make("new_owner");
    let deployment = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &deployment)?;
    // Store manager address, it will be used for querying
    let manager_addr = account.address()?;

    let sub_account = account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;

    let sub_accounts = account.sub_account_ids(None, None)?;
    assert_eq!(
        sub_accounts,
        SubAccountIdsResponse {
            // only one sub-account and it should be account_id 2
            sub_accounts: vec![2]
        }
    );

    sub_account.update_ownership(GovAction::TransferOwnership {
        new_owner: GovernanceDetails::Monarchy {
            monarch: new_owner.to_string(),
        },
        expiry: None,
    })?;

    // Make sure it's not updated until claimed
    let sub_accounts: SubAccountIdsResponse = chain.query(
        &abstract_std::account::QueryMsg::SubAccountIds {
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
    sub_account.call_as(&new_owner).execute(
        &abstract_std::account::ExecuteMsg::UpdateOwnership(ownership::GovAction::AcceptOwnership),
        &[],
    )?;

    // After claim it's updated
    let sub_accounts = account.sub_account_ids(None, None)?;
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
    Abstract::deploy_on_mock(chain.clone())?;
    abstract_integration_tests::account::account_move_ownership_to_sub_account(chain)?;
    Ok(())
}

#[test]
fn sub_account_move_ownership_to_sub_account() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &deployment)?;

    let sub_account = account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;
    let sub_account_addr = sub_account.address()?;

    let new_account = create_default_account(&sender, &deployment)?;

    let new_account_sub_account = new_account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;

    // sub-accounts state updated
    let sub_ids = new_account.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![4]);

    let new_governance = GovernanceDetails::SubAccount {
        account: sub_account_addr.to_string(),
    };
    new_account_sub_account.update_ownership(GovAction::TransferOwnership {
        new_owner: new_governance.clone(),
        expiry: None,
    })?;
    let new_account_sub_account_addr = new_account_sub_account.address()?;

    let mock_module = chain.addr_make("mock_module");

    // Should error as the ownership is not accepted yet
    new_account_sub_account
        .call_as(&sub_account_addr)
        .update_whitelist(vec![mock_module.to_string()], Vec::default())
        .expect_err("ownership not accepted yet.");

    sub_account.execute_msgs(
        vec![wasm_execute(
            new_account_sub_account_addr,
            &abstract_std::account::ExecuteMsg::<Empty>::UpdateOwnership(
                ownership::GovAction::AcceptOwnership,
            ),
            vec![],
        )?
        .into()],
        &[],
    )?;

    // sub-accounts state updated
    let sub_ids = sub_account.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![4]);
    // removed from the previous owner as well
    let sub_ids = new_account.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, Vec::<u32>::new());

    new_account_sub_account
        .call_as(&sub_account_addr)
        .update_whitelist(vec![mock_module.to_string()], Vec::default())?;

    new_account_sub_account.expect_whitelist(vec![mock_module])?;

    let info = new_account_sub_account.ownership()?;
    assert_eq!(new_governance, info.owner);
    take_storage_snapshot!(chain, "sub_account_move_ownership_to_sub_account");

    Ok(())
}

#[test]
fn account_updated_to_subaccount() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    // Creating account1
    let account_1 = create_default_account(&sender, &deployment)?;

    // Creating account2
    let account_2 = create_default_account(&sender, &deployment)?;

    // Setting account1 as pending owner of account2
    account_2.update_ownership(GovAction::TransferOwnership {
        new_owner: GovernanceDetails::SubAccount {
            account: account_1.addr_str()?,
        },
        expiry: None,
    })?;

    // account1 accepting account2 as a sub-account
    let accept_msg = abstract_std::account::ExecuteMsg::<Empty>::UpdateOwnership(
        ownership::GovAction::AcceptOwnership,
    );
    account_1.execute_msgs(
        vec![wasm_execute(account_2.addr_str()?, &accept_msg, vec![])?.into()],
        &[],
    )?;

    // Check account_1 knows about his new sub-account
    let ids = account_1.sub_account_ids(None, None)?;
    assert_eq!(ids.sub_accounts.len(), 1);
    Ok(())
}

#[test]
fn account_updated_to_subaccount_recursive() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    // Creating account1
    let account_1 = create_default_account(&sender, &deployment)?;

    // Creating account2
    let account_2 = create_default_account(&sender, &deployment)?;

    // Setting account1 as pending owner of account2
    account_2.update_ownership(GovAction::TransferOwnership {
        new_owner: GovernanceDetails::SubAccount {
            account: account_1.addr_str()?,
        },
        expiry: None,
    })?;
    // accepting ownership by sender instead of the manager
    account_2.update_ownership(ownership::GovAction::AcceptOwnership)?;

    // Check manager knows about his new sub-account
    let ids = account_1.sub_account_ids(None, None)?;
    assert_eq!(ids.sub_accounts.len(), 1);
    Ok(())
}

#[test]
fn top_level_owner() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    let account = create_default_account(&sender, &deployment)?;
    let sub_account = account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;

    let top_level_owner = sub_account.top_level_owner()?;
    assert_eq!(top_level_owner.address, sender);
    Ok(())
}

#[test]
fn cant_renounce_with_sub_accounts() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    let account = create_default_account(&sender, &deployment)?;
    // Creating sub account
    account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;

    let err: AccountError = account
        .update_ownership(ownership::GovAction::RenounceOwnership)
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, AccountError::RenounceWithSubAccount {});
    Ok(())
}

#[test]
fn can_renounce_sub_accounts() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    let account = create_default_account(&sender, &deployment)?;
    // Creating sub account
    let sub_account = account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;

    sub_account.update_ownership(ownership::GovAction::RenounceOwnership)?;

    account.update_ownership(ownership::GovAction::RenounceOwnership)?;

    // No owners
    // Renounced governance
    let account_owner = account.ownership()?;
    assert_eq!(account_owner.owner, GovernanceDetails::Renounced {});
    let sub_account_owner = sub_account.ownership()?;
    assert_eq!(sub_account_owner.owner, GovernanceDetails::Renounced {});

    Ok(())
}

#[test]
fn account_updated_to_subaccount_without_recursion() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    // Creating account1
    let account_1 = create_default_account(&sender, &deployment)?;

    // Creating account2
    let account_2 = create_default_account(&sender, &deployment)?;

    // Setting account1 as pending owner of account2
    account_2.update_ownership(GovAction::TransferOwnership {
        new_owner: GovernanceDetails::SubAccount {
            account: account_1.addr_str()?,
        },
        expiry: None,
    })?;

    // accepting ownership by sender instead of the manager
    account_1.execute_msgs(
        vec![WasmMsg::Execute {
            contract_addr: account_2.addr_str()?,
            msg: to_json_binary(
                &abstract_std::account::ExecuteMsg::<Empty>::UpdateOwnership(
                    GovAction::AcceptOwnership,
                ),
            )?,
            funds: Vec::default(),
        }
        .into()],
        &[],
    )?;

    // Check manager knows about his new sub-account
    let ids = account_1.sub_account_ids(None, None)?;
    assert_eq!(ids.sub_accounts.len(), 1);
    Ok(())
}

#[test]
fn sub_account_to_regular_account_without_recursion() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    // Creating account1
    let account = create_default_account(&sender, &deployment)?;
    let sub_account = account.create_and_return_sub_account(
        AccountDetails {
            name: "My subaccount".to_string(),
            ..Default::default()
        },
        &[],
    )?;

    account.execute_msgs(
        vec![WasmMsg::Execute {
            contract_addr: sub_account.addr_str()?,
            msg: to_json_binary(&abstract_account::msg::ExecuteMsg::UpdateOwnership(
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
        &[],
    )?;

    sub_account.update_ownership(GovAction::AcceptOwnership)?;
    let ownership = sub_account.ownership()?;
    assert_eq!(
        ownership.owner,
        GovernanceDetails::Monarchy {
            monarch: chain.sender_addr().to_string()
        }
    );
    Ok(())
}
