use abstract_integration_tests::{create_default_account, mock_modules, AResult};
use abstract_interface::{
    Abstract, AbstractAccount, Manager, ManagerExecFns, ManagerQueryFns, VCExecFns,
};
use abstract_manager::error::ManagerError;
use abstract_std::{
    manager::{
        ExecuteMsg as ManagerMsg, ModuleAddressesResponse, ModuleInstallConfig,
        QueryMsg as ManagerQuery,
    },
    objects::{account::TEST_ACCOUNT_ID, module::ModuleInfo},
    IBC_CLIENT,
};
use abstract_testing::prelude::TEST_NAMESPACE;
use anyhow::bail;
use cw_orch::{prelude::*, take_storage_snapshot};
use mock_modules::{adapter_1, deploy_modules, V1};
use speculoos::{assert_that, result::ResultAssertions};

pub fn ibc_client_installed<Chain: CwEnv>(manager: &Manager<Chain>) -> AResult {
    let ibc_addr = manager.module_addresses(vec![IBC_CLIENT.to_string()])?;
    if ibc_addr.modules.is_empty() {
        bail!("IBC client not installed")
    }
    Ok(())
}

#[test]
fn throws_if_enabling_when_already_enabled() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AbstractAccount { manager, proxy: _ } = &account;

    manager.update_settings(Some(true))?;
    let res = manager.update_settings(Some(true));

    assert_that!(&res).is_err();

    Ok(())
}

#[test]
fn throws_if_disabling_without_ibc_client_installed() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AbstractAccount { manager, proxy: _ } = &account;

    let res = manager.update_settings(Some(false));

    assert_that!(&res).is_err();

    Ok(())
}

#[test]
fn can_update_ibc_settings() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AbstractAccount { manager, proxy: _ } = &account;

    ibc_client_installed(manager).unwrap_err();
    manager.update_settings(Some(true))?;
    ibc_client_installed(manager)?;
    manager.update_settings(Some(false))?;
    ibc_client_installed(manager).unwrap_err();

    Ok(())
}
