use abstract_integration_tests::{create_default_account, AResult};
use abstract_interface::{Abstract, AbstractAccount, Manager, ManagerQueryFns};
use abstract_std::IBC_CLIENT;
use anyhow::bail;
use cw_orch::prelude::*;
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
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AbstractAccount { manager, proxy: _ } = &account;

    manager.ibc_enable(true)?;
    let res = manager.ibc_enable(true);

    assert_that!(&res).is_err();

    Ok(())
}

#[test]
fn throws_if_disabling_without_ibc_client_installed() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AbstractAccount { manager, proxy: _ } = &account;

    let res = manager.ibc_enable(false);

    assert_that!(&res).is_err();

    Ok(())
}

#[test]
fn can_update_ibc_settings() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AbstractAccount { manager, proxy: _ } = &account;

    ibc_client_installed(manager).unwrap_err();
    manager.ibc_enable(true)?;
    ibc_client_installed(manager)?;
    manager.ibc_enable(false)?;
    ibc_client_installed(manager).unwrap_err();

    Ok(())
}
