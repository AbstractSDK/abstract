use abstract_integration_tests::{create_default_account, AResult};
use abstract_interface::{Abstract, AbstractAccount, Manager, ManagerQueryFns};
use abstract_std::{
    manager::{ExecuteMsg as ManagerMsg, ModuleInstallConfig},
    objects::module::ModuleInfo,
    IBC_CLIENT,
};
use anyhow::bail;
use cw_orch::prelude::*;

pub fn ibc_client_installed<Chain: CwEnv>(manager: &Manager<Chain>) -> AResult {
    let ibc_addr = manager.module_addresses(vec![IBC_CLIENT.to_string()])?;
    if ibc_addr.modules.is_empty() {
        bail!("IBC client not installed")
    }
    Ok(())
}

#[test]
fn can_install_and_uninstall_ibc_client() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AbstractAccount { manager, proxy: _ } = &account;

    ibc_client_installed(manager).unwrap_err();
    manager.execute(
        &ManagerMsg::InstallModules {
            modules: vec![ModuleInstallConfig::new(
                ModuleInfo::from_id_latest(IBC_CLIENT)?,
                None,
            )],
        },
        &[],
    )?;
    ibc_client_installed(manager)?;
    manager.execute(
        &ManagerMsg::UninstallModule {
            module_id: IBC_CLIENT.to_string(),
        },
        &[],
    )?;
    ibc_client_installed(manager).unwrap_err();

    Ok(())
}
