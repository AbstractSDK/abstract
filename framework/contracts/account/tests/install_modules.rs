use abstract_account::error::AccountError;
use abstract_integration_tests::{create_default_account, mock_modules, AResult};
use abstract_interface::{Abstract, AccountI, VCExecFns};
use abstract_std::{
    account::{
        ExecuteMsg as ManagerMsg, ModuleAddressesResponse, ModuleInstallConfig,
        QueryMsg as ManagerQuery,
    },
    objects::{account::TEST_ACCOUNT_ID, module::ModuleInfo},
};
use abstract_testing::prelude::TEST_NAMESPACE;
use cw_orch::{prelude::*, take_storage_snapshot};
use mock_modules::{adapter_1, deploy_modules, V1};

#[test]
fn cannot_reinstall_module() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AccountI {
        account: manager,
        proxy: _,
    } = &account;

    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;

    deploy_modules(&chain);

    manager.execute(
        &ManagerMsg::InstallModules {
            modules: vec![ModuleInstallConfig::new(
                ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                None,
            )],
        },
        None,
    )?;

    let err = manager
        .execute(
            &ManagerMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                    None,
                )],
            },
            None,
        )
        .unwrap_err();
    let manager_err: AccountError = err.downcast().unwrap();
    assert_eq!(
        manager_err,
        AccountError::ModuleAlreadyInstalled(adapter_1::MOCK_ADAPTER_ID.to_owned())
    );
    Ok(())
}

#[test]
fn adds_module_to_account_modules() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AccountI {
        account: manager,
        proxy: _,
    } = &account;

    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;

    deploy_modules(&chain);

    manager.execute(
        &ManagerMsg::InstallModules {
            modules: vec![ModuleInstallConfig::new(
                ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                None,
            )],
        },
        None,
    )?;

    let addrs: ModuleAddressesResponse = manager.query(&ManagerQuery::ModuleAddresses {
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
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AccountI {
        account: manager,
        proxy: _,
    } = &account;

    let err = manager
        .execute(
            &ManagerMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
                    None,
                )],
            },
            None,
        )
        .unwrap_err();

    let manager_error: AccountError = err.downcast().unwrap();
    assert!(matches!(
        manager_error,
        AccountError::QueryModulesFailed { .. }
    ));
    Ok(())
}
