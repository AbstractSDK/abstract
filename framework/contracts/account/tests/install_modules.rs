use abstract_account::error::AccountError;
use abstract_integration_test_utils::{create_default_account, mock_modules, AResult};
use abstract_interface::{Abstract, AccountQueryFns, RegistryExecFns};
use abstract_std::{
    account::{
        ExecuteMsg as AccountMsg, ModuleAddressesResponse, ModuleInstallConfig,
        QueryMsg as AccountQuery,
    },
    objects::{module::ModuleInfo, ownership::GovOwnershipError},
};
use abstract_unit_test_utils::prelude::{TEST_ACCOUNT_ID, TEST_NAMESPACE};
use cw_orch::{prelude::*, take_storage_snapshot};
use mock_modules::{adapter_1, deploy_modules, V1};

#[test]
fn cannot_reinstall_module() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &abstr)?;

    abstr
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;

    deploy_modules(&chain);

    account.execute(
        &AccountMsg::InstallModules {
            modules: vec![ModuleInstallConfig::new(
                ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                None,
            )],
        },
        &[],
    )?;

    let err = account
        .execute(
            &AccountMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                    None,
                )],
            },
            &[],
        )
        .unwrap_err();
    let account_err: AccountError = err.downcast().unwrap();
    assert_eq!(
        account_err,
        AccountError::ModuleAlreadyInstalled(adapter_1::MOCK_ADAPTER_ID.to_owned())
    );
    Ok(())
}

#[test]
fn adds_module_to_account_modules() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &abstr)?;

    abstr
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;

    deploy_modules(&chain);

    account.execute(
        &AccountMsg::InstallModules {
            modules: vec![ModuleInstallConfig::new(
                ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                None,
            )],
        },
        &[],
    )?;

    let addrs: ModuleAddressesResponse = account.query(&AccountQuery::ModuleAddresses {
        ids: vec![adapter_1::MOCK_ADAPTER_ID.to_owned()],
    })?;
    assert_eq!(addrs.modules.len(), 1);
    take_storage_snapshot!(chain, "adds_module_to_account_modules");
    Ok(())
}

#[test]
fn useful_error_module_not_found() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &abstr)?;

    let err = account
        .execute(
            &AccountMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                    None,
                )],
            },
            &[],
        )
        .unwrap_err();

    let account_error: AccountError = err.downcast().unwrap();
    assert!(matches!(
        account_error,
        AccountError::QueryModulesFailed { .. }
    ));
    Ok(())
}

#[test]
fn only_admin_can_add_or_remove_module() -> AResult {
    let chain = MockBech32::new("mock");
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&chain.sender_addr(), &abstr)?;

    let not_admin = chain.addr_make("not_admin");
    let not_admin_error: AccountError = account
        .call_as(&not_admin)
        .execute(
            &AccountMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                    None,
                )],
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        not_admin_error,
        AccountError::Ownership(GovOwnershipError::NotOwner)
    );

    let not_admin_error: AccountError = account
        .call_as(&not_admin)
        .execute(
            &AccountMsg::UninstallModule {
                module_id: adapter_1::MOCK_ADAPTER_ID.to_string(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        not_admin_error,
        AccountError::Ownership(GovOwnershipError::NotOwner)
    );

    Ok(())
}

#[test]
fn fails_adding_previously_added_module() -> AResult {
    let chain = MockBech32::new("mock");
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&chain.sender_addr(), &abstr)?;

    abstr
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    // Deploy and install
    deploy_modules(&chain);
    account.execute(
        &AccountMsg::InstallModules {
            modules: vec![ModuleInstallConfig::new(
                ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                None,
            )],
        },
        &[],
    )?;

    let already_whitelisted: AccountError = account
        .execute(
            &AccountMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                    None,
                )],
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        already_whitelisted,
        AccountError::ModuleAlreadyInstalled(adapter_1::MOCK_ADAPTER_ID.to_string())
    );
    Ok(())
}

#[test]
fn remove_module() -> AResult {
    let chain = MockBech32::new("mock");
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&chain.sender_addr(), &abstr)?;

    abstr
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    // Deploy and install
    deploy_modules(&chain);
    account.execute(
        &AccountMsg::InstallModules {
            modules: vec![ModuleInstallConfig::new(
                ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                None,
            )],
        },
        &[],
    )?;

    let module_infos = account.module_infos(None, None)?.module_infos;
    assert_eq!(module_infos[0].id, adapter_1::MOCK_ADAPTER_ID);

    account.execute(
        &AccountMsg::UninstallModule {
            module_id: adapter_1::MOCK_ADAPTER_ID.to_string(),
        },
        &[],
    )?;
    let module_infos = account.module_infos(None, None)?.module_infos;
    assert!(module_infos.is_empty());

    Ok(())
}

#[test]
fn fails_removing_non_existing_module() -> AResult {
    let chain = MockBech32::new("mock");
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&chain.sender_addr(), &abstr)?;

    let err: AccountError = account
        .execute(
            &AccountMsg::UninstallModule {
                module_id: adapter_1::MOCK_ADAPTER_ID.to_string(),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        err,
        AccountError::ModuleNotFound(adapter_1::MOCK_ADAPTER_ID.to_string())
    );
    Ok(())
}
