use abstract_account::error::AccountError;
use abstract_integration_tests::{create_default_account, mock_modules, AResult};
use abstract_interface::{Abstract, VCExecFns};
use abstract_std::{
    account::{
        ExecuteMsg as AccountMsg, ModuleAddressesResponse, ModuleInstallConfig,
        QueryMsg as AccountQuery,
    },
    objects::module::ModuleInfo,
};
use abstract_testing::prelude::{mock_bech32_admin, TEST_ACCOUNT_ID, TEST_NAMESPACE};
use cw_orch::{prelude::*, take_storage_snapshot};
use mock_modules::{adapter_1, deploy_modules, V1};

#[test]
fn cannot_reinstall_module() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on(chain.clone(), mock_bech32_admin(&chain))?;
    let account = create_default_account(&sender, &abstr)?;

    abstr
        .version_control
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
    let abstr = Abstract::deploy_on(chain.clone(), mock_bech32_admin(&chain))?;
    let account = create_default_account(&sender, &abstr)?;

    abstr
        .version_control
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
    let abstr = Abstract::deploy_on(chain.clone(), mock_bech32_admin(&chain))?;
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
