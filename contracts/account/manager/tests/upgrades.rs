mod common;

use abstract_app::mock::{MockInitMsg, MockMigrateMsg};
use abstract_core::{
    app::{self, BaseInstantiateMsg},
    objects::module::{ModuleInfo, ModuleVersion},
    AbstractError,
};
use abstract_interface::{Abstract, AbstractAccount, Manager, ManagerExecFns, VCExecFns};

use abstract_manager::error::ManagerError;
use abstract_testing::addresses::{TEST_ACCOUNT_ID, TEST_NAMESPACE};

use common::mock_modules::*;
use common::{create_default_account, AResult};
use cosmwasm_std::to_binary;
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use speculoos::prelude::*;

fn install_module_version(
    manager: &Manager<Mock>,
    abstr: &Abstract<Mock>,
    module: &str,
    version: &str,
) -> anyhow::Result<String> {
    manager.install_module_version(
        module,
        ModuleVersion::Version(version.to_string()),
        &app::InstantiateMsg {
            module: MockInitMsg,
            base: BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.addr_str()?,
            },
        },
        None,
    )?;

    Ok(manager.module_info(module)?.unwrap().address.to_string())
}

#[test]
fn install_app_successful() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;
    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // dependency for mock_adapter1 not met
    let res = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string()).contains(
        "module tester:mock-adapter1 is a dependency of tester:mock-app1 and is not installed.",
    );

    // install adapter 1
    let adapter1 = install_module_version(manager, &abstr, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // second dependency still not met
    let res = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string()).contains(
        "module tester:mock-adapter2 is a dependency of tester:mock-app1 and is not installed.",
    );

    // install adapter 2
    let adapter2 = install_module_version(manager, &abstr, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;

    account.expect_modules(vec![adapter1, adapter2, app1])?;
    Ok(())
}

#[test]
fn install_app_versions_not_met() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;
    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // install adapter 2
    let _adapter2 = install_module_version(manager, &abstr, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // successfully install app 1
    let _app1 = install_module_version(manager, &abstr, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // attempt to install app with version 2

    let res = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V2);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string())
        .contains("Module tester:mock-adapter1 with version 1.0.0 does not fit requirement ^2.0.0");
    Ok(())
}

#[test]
fn upgrade_app() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;
    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // install adapter 1
    let adapter1 = install_module_version(manager, &abstr, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // install adapter 2
    let adapter2 = install_module_version(manager, &abstr, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;
    account.expect_modules(vec![adapter1, adapter2, app1])?;

    // attempt upgrade app 1 to version 2
    let res = manager.upgrade_module(
        app_1::MOCK_APP_ID,
        &app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: MockMigrateMsg,
        },
    );
    // fails because adapter 1 is not version 2
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::VersionRequirementNotMet {
            module_id: adapter_1::MOCK_ADAPTER_ID.into(),
            version: V1.into(),
            comp: "^2.0.0".into(),
            post_migration: true,
        }
        .to_string(),
    );

    // upgrade adapter 1 to version 2
    let res = manager.upgrade_module(
        adapter_1::MOCK_ADAPTER_ID,
        &app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: Empty {},
        },
    );
    // fails because app v1 is not version 2 and depends on adapter 1 being version 1.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::VersionRequirementNotMet {
            module_id: adapter_1::MOCK_ADAPTER_ID.into(),
            version: V2.into(),
            comp: "^1.0.0".into(),
            post_migration: false,
        }
        .to_string(),
    );

    // solution: upgrade multiple modules in the same tx
    // Important: the order of the modules is important. The hightest-level dependents must be migrated first.

    // attempt to upgrade app 1 to identical version while updating other modules
    let res = manager.upgrade(vec![
        (
            ModuleInfo::from_id(app_1::MOCK_APP_ID, ModuleVersion::Version(V1.to_string()))?,
            Some(to_binary(&app::MigrateMsg {
                base: app::BaseMigrateMsg {},
                module: MockMigrateMsg,
            })?),
        ),
        (
            ModuleInfo::from_id_latest(adapter_1::MOCK_ADAPTER_ID)?,
            None,
        ),
        (
            ModuleInfo::from_id_latest(adapter_2::MOCK_ADAPTER_ID)?,
            None,
        ),
    ]);

    // fails because app v1 is depends on adapter 1 being version 1.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::Abstract(AbstractError::CannotDowngradeContract {
            contract: app_1::MOCK_APP_ID.into(),
            from: V1.parse().unwrap(),
            to: V1.parse().unwrap(),
        })
        .to_string(),
    );

    // attempt to upgrade app 1 to version 2 while not updating other modules
    let res = manager.upgrade(vec![(
        ModuleInfo::from_id(app_1::MOCK_APP_ID, ModuleVersion::Version(V2.to_string()))?,
        Some(to_binary(&app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: MockMigrateMsg,
        })?),
    )]);

    // fails because app v1 is depends on adapter 1 being version 2.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::VersionRequirementNotMet {
            module_id: adapter_1::MOCK_ADAPTER_ID.into(),
            version: V1.into(),
            comp: "^2.0.0".into(),
            post_migration: true,
        }
        .to_string(),
    );

    // attempt to upgrade app 1 to identical version while updating other modules
    let res = manager.upgrade(vec![
        (
            ModuleInfo::from_id(app_1::MOCK_APP_ID, ModuleVersion::Version(V2.to_string()))?,
            Some(to_binary(&app::MigrateMsg {
                base: app::BaseMigrateMsg {},
                module: MockMigrateMsg,
            })?),
        ),
        (
            ModuleInfo::from_id(
                adapter_1::MOCK_ADAPTER_ID,
                ModuleVersion::Version(V1.to_string()),
            )?,
            None,
        ),
        (
            ModuleInfo::from_id(
                adapter_2::MOCK_ADAPTER_ID,
                ModuleVersion::Version(V1.to_string()),
            )?,
            None,
        ),
    ]);

    // fails because app v1 is depends on adapter 1 being version 2.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::VersionRequirementNotMet {
            module_id: adapter_1::MOCK_ADAPTER_ID.into(),
            version: V1.into(),
            comp: "^2.0.0".into(),
            post_migration: true,
        }
        .to_string(),
    );

    // successfully upgrade all the modules
    manager.upgrade(vec![
        (
            ModuleInfo::from_id_latest(app_1::MOCK_APP_ID)?,
            Some(to_binary(&app::MigrateMsg {
                base: app::BaseMigrateMsg {},
                module: MockMigrateMsg,
            })?),
        ),
        (
            ModuleInfo::from_id_latest(adapter_1::MOCK_ADAPTER_ID)?,
            None,
        ),
        (
            ModuleInfo::from_id_latest(adapter_2::MOCK_ADAPTER_ID)?,
            None,
        ),
    ])?;

    Ok(())
}

#[test]
fn uninstall_modules() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;
    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    let adapter1 = install_module_version(manager, &abstr, adapter_1::MOCK_ADAPTER_ID, V1)?;
    let adapter2 = install_module_version(manager, &abstr, adapter_2::MOCK_ADAPTER_ID, V1)?;
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;
    account.expect_modules(vec![adapter1, adapter2, app1])?;

    let res = manager.uninstall_module(adapter_1::MOCK_ADAPTER_ID.to_string());
    // fails because app is depends on adapter 1
    assert_that!(res.unwrap_err().root().to_string())
        .contains(ManagerError::ModuleHasDependents(vec![app_1::MOCK_APP_ID.into()]).to_string());
    // same for adapter 2
    let res = manager.uninstall_module(adapter_2::MOCK_ADAPTER_ID.to_string());
    assert_that!(res.unwrap_err().root().to_string())
        .contains(ManagerError::ModuleHasDependents(vec![app_1::MOCK_APP_ID.into()]).to_string());

    // we can only uninstall if the app is uninstalled first
    manager.uninstall_module(app_1::MOCK_APP_ID.to_string())?;
    // now we can uninstall adapter 1
    manager.uninstall_module(adapter_1::MOCK_ADAPTER_ID.to_string())?;
    // and adapter 2
    manager.uninstall_module(adapter_2::MOCK_ADAPTER_ID.to_string())?;
    Ok(())
}

#[test]
fn update_adapter_with_authorized_addrs() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy } = &account;
    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // install adapter 1
    let adapter1 = install_module_version(manager, &abstr, adapter_1::MOCK_ADAPTER_ID, V1)?;
    account.expect_modules(vec![adapter1.clone()])?;

    // register an authorized address on Adapter 1
    let authorizee = "authorizee";
    manager.update_adapter_authorized_addresses(
        adapter_1::MOCK_ADAPTER_ID,
        vec![authorizee.to_string()],
        vec![],
    )?;

    // upgrade adapter 1 to version 2
    manager.upgrade_module(
        adapter_1::MOCK_ADAPTER_ID,
        &app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: Empty {},
        },
    )?;
    use abstract_core::manager::QueryMsgFns as _;
    let adapter_v2 = manager.module_addresses(vec![adapter_1::MOCK_ADAPTER_ID.into()])?;
    // assert that the address actually changed
    assert_that!(adapter_v2.modules[0].1).is_not_equal_to(Addr::unchecked(adapter1.clone()));

    let adapter = adapter_1::BootMockAdapter1V2::new_test(chain);
    use abstract_core::adapter::BaseQueryMsgFns as _;
    let authorized = adapter.authorized_addresses(proxy.addr_str()?)?;
    assert_that!(authorized.addresses).contains(Addr::unchecked(authorizee));

    // assert that authorized address was removed from old Adapter
    adapter.set_address(&Addr::unchecked(adapter1));
    let authorized = adapter.authorized_addresses(proxy.addr_str()?)?;
    assert_that!(authorized.addresses).is_empty();
    Ok(())
}

/*#[test]
fn upgrade_manager_last() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), Empty {})?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;

    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, vec![TEST_NAMESPACE.to_string()])?;
    deploy_modules(&chain);

    // install adapter 1
    let adapter1 = install_module_version(manager, &abstr, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // install adapter 2
    let adapter2 = install_module_version(manager, &abstr, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;
    account.expect_modules(vec![adapter1, adapter2, app1])?;

    // Upgrade all modules, including the manager module, but ensure the manager is upgraded last
    let res = manager.upgrade(vec![
        (
            ModuleInfo::from_id_latest(app_1::MOCK_APP_ID)?,
            Some(to_binary(&app::MigrateMsg {
                base: app::BaseMigrateMsg {},
                module: MockMigrateMsg,
            })?),
        ),
        (
            ModuleInfo::from_id_latest("abstract:manager")?,
            Some(to_binary(&manager::MigrateMsg {})?),
        ),
        (ModuleInfo::from_id_latest(adapter_1::MOCK_ADAPTER_ID)?, None),
        (ModuleInfo::from_id_latest(adapter_2::MOCK_ADAPTER_ID)?, None),
    ])?;

    // get the events
    let mut events: Vec<Event> = res.events;
    events.pop();
    let migrate_event = events.pop().unwrap();

    // the 2nd last event will be the manager execution
    assert_that!(migrate_event.attributes).has_length(3);
    let mut attributes = migrate_event.attributes;
    // check that the action was migrate
    assert_that!(attributes.pop())
        .is_some()
        .is_equal_to(Attribute::from(("action", "migrate")));

    // and that it was the manager
    assert_that!(attributes.pop())
        .is_some()
        .is_equal_to(Attribute::from(("contract", "abstract:manager")));

    Ok(())
}*/

#[test]
fn no_duplicate_migrations() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), Empty {})?;

    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;

    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // Install adapter 1
    let adapter1 = install_module_version(manager, &abstr, adapter_1::MOCK_ADAPTER_ID, V1)?;

    account.expect_modules(vec![adapter1])?;

    // Upgrade all modules, including the manager module
    let res = manager.upgrade(vec![
        (
            ModuleInfo::from_id_latest(adapter_1::MOCK_ADAPTER_ID)?,
            None,
        ),
        (
            ModuleInfo::from_id_latest(adapter_1::MOCK_ADAPTER_ID)?,
            None,
        ),
    ]);

    assert_that!(res).is_err();

    assert_that!(res.unwrap_err().root().to_string()).is_equal_to(
        ManagerError::DuplicateModuleMigration {
            module_id: adapter_1::MOCK_ADAPTER_ID.to_string(),
        }
        .to_string(),
    );

    Ok(())
}

// TODO:
// - adapter-adapter dependencies
// - app-adapter dependencies
// - app-app dependencies
