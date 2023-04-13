mod common;

use abstract_app::mock::{MockInitMsg, MockMigrateMsg};
use abstract_boot::{Abstract, AbstractAccount, Manager, ManagerExecFns, VCExecFns};
use abstract_core::app::{self, BaseInstantiateMsg};
use abstract_core::objects::module::{ModuleInfo, ModuleVersion};
use abstract_manager::error::ManagerError;
use abstract_testing::prelude::TEST_VERSION;
use boot_core::{instantiate_default_mock_env, Addr, ContractInstance, Deploy, Empty, Mock};
use common::mock_modules::*;
use common::{create_default_account, AResult};
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
            module: MockInitMsg,
            base: BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.addr_str()?,
            },
        },
    )?;

    Ok(manager.module_info(module)?.unwrap().address)
}

#[test]
fn install_app_successful() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;
    abstr
        .version_control
        .claim_namespaces(0, vec!["tester".to_string()])?;
    deploy_modules(&chain);

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

    account.expect_modules(vec![api1, api2, app1])?;
    Ok(())
}

#[test]
fn install_app_versions_not_met() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;
    abstr
        .version_control
        .claim_namespaces(0, vec!["tester".to_string()])?;
    deploy_modules(&chain);

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
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;
    abstr
        .version_control
        .claim_namespaces(0, vec!["tester".to_string()])?;
    deploy_modules(&chain);

    // install api 1
    let api1 = install_module_version(manager, &abstr, api_1::MOCK_API_ID, V1)?;

    // install api 2
    let api2 = install_module_version(manager, &abstr, api_2::MOCK_API_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;
    account.expect_modules(vec![api1, api2, app1])?;

    // attempt upgrade app 1 to version 2
    let res = manager.upgrade_module(
        app_1::MOCK_APP_ID,
        &app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: MockMigrateMsg,
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
            module: Empty {},
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
                module: MockMigrateMsg,
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
            module: MockMigrateMsg,
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
                module: MockMigrateMsg,
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
                module: MockMigrateMsg,
            })?),
        ),
        (ModuleInfo::from_id_latest(api_1::MOCK_API_ID)?, None),
        (ModuleInfo::from_id_latest(api_2::MOCK_API_ID)?, None),
    ])?;

    Ok(())
}

#[test]
fn uninstall_modules() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;
    abstr
        .version_control
        .claim_namespaces(0, vec!["tester".to_string()])?;
    deploy_modules(&chain);

    let api1 = install_module_version(manager, &abstr, api_1::MOCK_API_ID, V1)?;
    let api2 = install_module_version(manager, &abstr, api_2::MOCK_API_ID, V1)?;
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;
    account.expect_modules(vec![api1, api2, app1])?;

    let res = manager.uninstall_module(api_1::MOCK_API_ID.to_string());
    // fails because app is depends on api 1
    assert_that!(res.unwrap_err().root().to_string())
        .contains(ManagerError::ModuleHasDependents(vec![app_1::MOCK_APP_ID.into()]).to_string());
    // same for api 2
    let res = manager.uninstall_module(api_2::MOCK_API_ID.to_string());
    assert_that!(res.unwrap_err().root().to_string())
        .contains(ManagerError::ModuleHasDependents(vec![app_1::MOCK_APP_ID.into()]).to_string());

    // we can only uninstall if the app is uninstalled first
    manager.uninstall_module(app_1::MOCK_APP_ID.to_string())?;
    // now we can uninstall api 1
    manager.uninstall_module(api_1::MOCK_API_ID.to_string())?;
    // and api 2
    manager.uninstall_module(api_2::MOCK_API_ID.to_string())?;
    Ok(())
}

#[test]
fn update_api_with_authorized_addrs() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let abstr = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse()?)?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy } = &account;
    abstr
        .version_control
        .claim_namespaces(0, vec!["tester".to_string()])?;
    deploy_modules(&chain);

    // install api 1
    let api1 = install_module_version(manager, &abstr, api_1::MOCK_API_ID, V1)?;
    account.expect_modules(vec![api1.clone()])?;

    // register a authorized address on API1
    manager.update_api_authorized_addresses(
        api_1::MOCK_API_ID,
        vec!["authorizee".to_string()],
        vec![],
    )?;

    // upgrade api 1 to version 2
    manager.upgrade_module(
        api_1::MOCK_API_ID,
        &app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: Empty {},
        },
    )?;
    use abstract_core::manager::QueryMsgFns as _;
    let api_v2 = manager.module_addresses(vec![api_1::MOCK_API_ID.into()])?;
    // assert that the address actually changed
    assert_that!(api_v2.modules[0].1).is_not_equal_to(api1.clone());

    let api = api_1::BootMockApi1V2::new(chain);
    use abstract_core::api::BaseQueryMsgFns as _;
    let authorized = api.authorized_addresses(proxy.addr_str()?)?;
    assert_that!(authorized.addresses).contains(Addr::unchecked("authorizee"));

    // assert that authorized address was removed from old API
    api.set_address(&Addr::unchecked(api1));
    let authorized = api.authorized_addresses(proxy.addr_str()?)?;
    assert_that!(authorized.addresses).is_empty();
    Ok(())
}

// TODO:
// - api-api dependencies
// - app-api dependencies
// - app-app dependencies
