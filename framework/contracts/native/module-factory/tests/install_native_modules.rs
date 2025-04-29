use abstract_integration_tests::{create_default_account, AResult};
use abstract_interface::{Abstract, AccountI, AccountQueryFns};
use abstract_std::{
    account::{ExecuteMsg as AccountMsg, ModuleInstallConfig},
    objects::module::ModuleInfo,
    IBC_CLIENT,
};
use anyhow::bail;
use cw_orch::prelude::*;

pub fn ibc_client_installed<Chain: CwEnv>(account: &AccountI<Chain>) -> AResult {
    let ibc_addr = account.module_addresses(vec![IBC_CLIENT.to_string()])?;
    if ibc_addr.modules.is_empty() {
        bail!("IBC client not installed")
    }
    Ok(())
}

#[test]
fn can_install_and_uninstall_ibc_client() -> AResult {
    let chain = MockBech32::new("mock");
    let abstr = Abstract::deploy_on(chain.clone(), ())?;
    let account = create_default_account(&chain.sender_addr(), &abstr)?;

    ibc_client_installed(&account).unwrap_err();
    account.execute(
        &AccountMsg::InstallModules {
            modules: vec![ModuleInstallConfig::new(
                ModuleInfo::from_id_latest(IBC_CLIENT)?,
                None,
            )],
        },
        &[],
    )?;
    ibc_client_installed(&account)?;
    account.execute(
        &AccountMsg::UninstallModule {
            module_id: IBC_CLIENT.to_string(),
        },
        &[],
    )?;
    ibc_client_installed(&account).unwrap_err();

    Ok(())
}
