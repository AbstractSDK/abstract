mod common;

use abstract_core::{
    manager::{
        ExecuteMsg as ManagerMsg, ModuleAddressesResponse, ModuleInstallConfig,
        QueryMsg as ManagerQuery,
    },
    objects::{account::TEST_ACCOUNT_ID, module::ModuleInfo},
};
use abstract_interface::{Abstract, AbstractAccount, VCExecFns};
use abstract_manager::error::ManagerError;
use abstract_testing::prelude::TEST_NAMESPACE;
use common::{
    mock_modules::{adapter_1, deploy_modules, V1},
    *,
};
use cosmwasm_std::Addr;
use cw_orch::{
    deploy::Deploy,
    prelude::{CwOrchExecute, CwOrchQuery, Mock},
    take_storage_snapshot,
};

#[test]
fn cannot_reinstall_module() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AbstractAccount { manager, proxy: _ } = &account;

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
    let manager_err: ManagerError = err.downcast().unwrap();
    assert_eq!(
        manager_err,
        ManagerError::ModuleAlreadyInstalled(adapter_1::MOCK_ADAPTER_ID.to_owned())
    );
    Ok(())
}

#[test]
fn adds_module_to_account_modules() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;

    let AbstractAccount { manager, proxy: _ } = &account;

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
