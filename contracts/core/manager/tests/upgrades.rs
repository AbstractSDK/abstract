mod common;

use abstract_app::mock::{MockInitMsg, MockMigrateMsg};
use abstract_boot::{Abstract, AbstractBootError, Manager, ManagerExecFns, OS};
use abstract_manager::error::ManagerError;
use abstract_os::app::{self, BaseInstantiateMsg};
use abstract_os::objects::module::{ModuleInfo, ModuleVersion};
use abstract_testing::prelude::TEST_VERSION;
use boot_core::{
    instantiate_default_mock_env, Addr, BootError, ContractInstance, Deploy, Empty, Mock,
};
use common::mock_modules::*;
use common::{create_default_os, init_abstract_env, init_mock_api, AResult, TEST_COIN};
use cosmwasm_std::to_binary;
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
            app: MockInitMsg,
            base: BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.addr_str()?,
            },
        },
    )?;

    Ok(manager.module_info(module)?.unwrap().address)
}

#[test]
fn install_app_successful() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    deploy_modules(&chain);
    let os = create_default_os(&abstr.os_factory)?;
    let OS { manager, proxy: _ } = &os;

    // dependency for mock_api1 not met
    let res = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string()).contains(
        "module tester:mock-api1 is a dependency of tester:mock-app1 and is not installed.",
    );

    // install api 1
    let api1 = install_module_version(manager, &abstr, api_1::MOCK_API_ID, V1)?;

    // second dependency still not met
    let res = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string()).contains(
        "module tester:mock-api2 is a dependency of tester:mock-app1 and is not installed.",
    );

    // install api 2
    let api2 = install_module_version(manager, &abstr, api_2::MOCK_API_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;

    os.expect_modules(vec![api1, api2, app1])?;
    Ok(())
}

#[test]
fn install_app_versions_not_met() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    deploy_modules(&chain);
    let os = create_default_os(&abstr.os_factory)?;
    let OS { manager, proxy: _ } = &os;

    // install api 2
    let _api2 = install_module_version(manager, &abstr, api_1::MOCK_API_ID, V1)?;

    // successfully install app 1
    let _app1 = install_module_version(manager, &abstr, api_2::MOCK_API_ID, V1)?;

    // attempt to install app with version 2

    let res = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V2);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string())
        .contains("Module tester:mock-api1 with version 1.0.0 does not fit requirement ^2.0.0");
    Ok(())
}

#[test]
fn upgrade_app_() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    deploy_modules(&chain);
    let os = create_default_os(&abstr.os_factory)?;
    let OS { manager, proxy: _ } = &os;

    // install api 1
    let api1 = install_module_version(manager, &abstr, api_1::MOCK_API_ID, V1)?;

    // install api 2
    let api2 = install_module_version(manager, &abstr, api_2::MOCK_API_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;
    os.expect_modules(vec![api1, api2, app1])?;

    // attempt upgrade app 1 to version 2
    let res = manager.upgrade_module(
        app_1::MOCK_APP_ID,
        &app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            app: MockMigrateMsg,
        },
    );
    // fails because api 1 is not version 2
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::VersionRequirementNotMet {
            module_id: api_1::MOCK_API_ID.into(),
            version: V1.into(),
            comp: "^2.0.0".into(),
            post_migration: true,
        }
        .to_string(),
    );

    // upgrade api 1 to version 2

    let res = manager.upgrade_module(
        api_1::MOCK_API_ID,
        &app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            app: Empty {},
        },
    );
    // fails because app v1 is not version 2 and depends on api 1 being version 1.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::VersionRequirementNotMet {
            module_id: api_1::MOCK_API_ID.into(),
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
                app: MockMigrateMsg,
            })?),
        ),
        (ModuleInfo::from_id_latest(api_1::MOCK_API_ID)?, None),
        (ModuleInfo::from_id_latest(api_2::MOCK_API_ID)?, None),
    ]);

    // fails because app v1 is depends on api 1 being version 1.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::VersionRequirementNotMet {
            module_id: api_1::MOCK_API_ID.into(),
            version: V2.into(),
            comp: "^1.0.0".into(),
            post_migration: true,
        }
        .to_string(),
    );

    // attempt to upgrade app 1 to version 2 while not updating other modules
    let res = manager.upgrade(vec![(
        ModuleInfo::from_id(app_1::MOCK_APP_ID, ModuleVersion::Version(V2.to_string()))?,
        Some(to_binary(&app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            app: MockMigrateMsg,
        })?),
    )]);

    // fails because app v1 is depends on api 1 being version 2.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::VersionRequirementNotMet {
            module_id: api_1::MOCK_API_ID.into(),
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
                app: MockMigrateMsg,
            })?),
        ),
        (
            ModuleInfo::from_id(api_1::MOCK_API_ID, ModuleVersion::Version(V1.to_string()))?,
            None,
        ),
        (
            ModuleInfo::from_id(api_2::MOCK_API_ID, ModuleVersion::Version(V1.to_string()))?,
            None,
        ),
    ]);

    // fails because app v1 is depends on api 1 being version 2.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        ManagerError::VersionRequirementNotMet {
            module_id: api_1::MOCK_API_ID.into(),
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
                app: MockMigrateMsg,
            })?),
        ),
        (ModuleInfo::from_id_latest(api_1::MOCK_API_ID)?, None),
        (ModuleInfo::from_id_latest(api_2::MOCK_API_ID)?, None),
    ])?;

    Ok(())
}

#[test]
fn uninstall_modules() -> AResult {
    let sender = Addr::unchecked(common::ROOT_USER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    deploy_modules(&chain);
    let os = create_default_os(&abstr.os_factory)?;
    let OS { manager, proxy: _ } = &os;
    let api1 = install_module_version(manager, &abstr, api_1::MOCK_API_ID, V1)?;
    let api2 = install_module_version(manager, &abstr, api_2::MOCK_API_ID, V1)?;
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;
    os.expect_modules(vec![api1, api2, app1])?;

    let res = manager.uninstall_module(api_1::MOCK_API_ID);
    // fails because app is depends on api 1
    assert_that!(res.unwrap_err().root().to_string())
        .contains(ManagerError::ModuleHasDependents(vec![app_1::MOCK_APP_ID.into()]).to_string());
    // same for api 2
    let res = manager.uninstall_module(api_2::MOCK_API_ID);
    assert_that!(res.unwrap_err().root().to_string())
        .contains(ManagerError::ModuleHasDependents(vec![app_1::MOCK_APP_ID.into()]).to_string());

    // we can only uninstall if the app is uninstalled first
    manager.uninstall_module(app_1::MOCK_APP_ID)?;
    // now we can uninstall api 1
    manager.uninstall_module(api_1::MOCK_API_ID)?;
    // and api 2
    manager.uninstall_module(api_2::MOCK_API_ID)?;
    Ok(())
}
