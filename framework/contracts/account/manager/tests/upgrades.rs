mod common;

use abstract_app::mock::{MockInitMsg, MockMigrateMsg};
use abstract_core::{
    app::{self, BaseInstantiateMsg},
    manager::ModuleVersionsResponse,
    module_factory::{ModuleInstallConfig, SimulateInstallModulesResponse},
    objects::{
        fee::FixedFee,
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion, Monetization},
        module_reference::ModuleReference,
        namespace::Namespace,
        AccountId,
    },
    version_control::UpdateModule,
    AbstractError,
};
use abstract_interface::{
    Abstract, AbstractAccount, AbstractInterfaceError, AccountDetails, MFactoryQueryFns, Manager,
    ManagerExecFns, ManagerQueryFns, VCExecFns,
};

use abstract_manager::error::ManagerError;
use abstract_testing::addresses::{TEST_ACCOUNT_ID, TEST_NAMESPACE};

use common::mock_modules::*;
use common::{create_default_account, AResult};
use cosmwasm_std::{coin, coins, to_binary, Uint128};
use cw2::ContractVersion;
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
                version_control_address: abstr.version_control.addr_str()?,
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
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
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
        // Error from macro
        "no address",
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
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
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
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
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
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
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
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
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
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
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
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;

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

#[test]
fn create_account_with_installed_module() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let factory = &deployment.account_factory;

    let _deployer_acc = factory.create_new_account(
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            base_asset: None,
            install_modules: vec![],
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        None,
    )?;
    deploy_modules(&chain);

    let account = factory
        .create_new_account(
            AccountDetails {
                name: String::from("second_account"),
                description: Some(String::from("account_description")),
                link: Some(String::from("https://account_link_of_at_least_11_char")),
                namespace: None,
                base_asset: None,
                install_modules: vec![
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            adapter_1::MOCK_ADAPTER_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        None,
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            adapter_2::MOCK_ADAPTER_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        None,
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            app_1::MOCK_APP_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        Some(to_binary(&app::InstantiateMsg {
                            module: MockInitMsg,
                            base: BaseInstantiateMsg {
                                ans_host_address: deployment.ans_host.addr_str()?,
                                version_control_address: deployment.version_control.addr_str()?,
                            },
                        })?),
                    ),
                ],
            },
            GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            None,
        )
        .unwrap();

    // Make sure all installed
    let account_module_versions = account.manager.module_versions(vec![
        String::from(adapter_1::MOCK_ADAPTER_ID),
        String::from(adapter_2::MOCK_ADAPTER_ID),
        String::from(app_1::MOCK_APP_ID),
    ])?;
    assert_eq!(
        account_module_versions,
        ModuleVersionsResponse {
            versions: vec![
                ContractVersion {
                    contract: String::from(adapter_1::MOCK_ADAPTER_ID),
                    version: String::from(V1)
                },
                ContractVersion {
                    contract: String::from(adapter_2::MOCK_ADAPTER_ID),
                    version: String::from(V1)
                },
                ContractVersion {
                    contract: String::from(app_1::MOCK_APP_ID),
                    version: String::from(V1)
                }
            ]
        }
    );
    Ok(())
}

#[test]
fn create_sub_account_with_installed_module() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let factory = &deployment.account_factory;

    let _deployer_acc = factory.create_new_account(
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            base_asset: None,
            install_modules: vec![],
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        None,
    )?;
    deploy_modules(&chain);

    _deployer_acc.manager.create_sub_account(
        vec![
            ModuleInstallConfig::new(
                ModuleInfo::from_id(
                    adapter_1::MOCK_ADAPTER_ID,
                    ModuleVersion::Version(V1.to_owned()),
                )?,
                None,
            ),
            ModuleInstallConfig::new(
                ModuleInfo::from_id(
                    adapter_2::MOCK_ADAPTER_ID,
                    ModuleVersion::Version(V1.to_owned()),
                )?,
                None,
            ),
            ModuleInstallConfig::new(
                ModuleInfo::from_id(app_1::MOCK_APP_ID, ModuleVersion::Version(V1.to_owned()))?,
                Some(to_binary(&app::InstantiateMsg {
                    module: MockInitMsg,
                    base: BaseInstantiateMsg {
                        ans_host_address: deployment.ans_host.addr_str()?,
                        version_control_address: deployment.ans_host.addr_str()?,
                    },
                })?),
            ),
        ],
        String::from("sub_account"),
        None,
        Some(String::from("account_description")),
        None,
        None,
        &[],
    )?;

    let account = AbstractAccount::new(&deployment, Some(AccountId::local(2)));

    // Make sure all installed
    let account_module_versions = account.manager.module_versions(vec![
        String::from(adapter_1::MOCK_ADAPTER_ID),
        String::from(adapter_2::MOCK_ADAPTER_ID),
        String::from(app_1::MOCK_APP_ID),
    ])?;
    assert_eq!(
        account_module_versions,
        ModuleVersionsResponse {
            versions: vec![
                ContractVersion {
                    contract: String::from(adapter_1::MOCK_ADAPTER_ID),
                    version: String::from(V1)
                },
                ContractVersion {
                    contract: String::from(adapter_2::MOCK_ADAPTER_ID),
                    version: String::from(V1)
                },
                ContractVersion {
                    contract: String::from(app_1::MOCK_APP_ID),
                    version: String::from(V1)
                }
            ]
        }
    );
    Ok(())
}

#[test]
fn create_account_with_installed_module_and_monetization() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    // Adding coins to fill monetization
    chain.add_balance(&sender, vec![coin(10, "coin1"), coin(10, "coin2")])?;
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let factory = &deployment.account_factory;

    let _deployer_acc = factory.create_new_account(
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            base_asset: None,
            install_modules: vec![],
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        None,
    )?;
    deploy_modules(&chain);
    // Add monetization
    deployment.version_control.update_module_configuration(
        "mock-adapter1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin1")))),
            instantiation_funds: None,
        },
    )?;
    deployment.version_control.update_module_configuration(
        "mock-adapter2".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin1")))),
            instantiation_funds: None,
        },
    )?;
    deployment.version_control.update_module_configuration(
        "mock-app1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin2")))),
            instantiation_funds: None,
        },
    )?;
    // Check how much we need
    let simulate_response = deployment.module_factory.simulate_install_modules(vec![
        ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
        ModuleInfo::from_id(adapter_2::MOCK_ADAPTER_ID, V1.into()).unwrap(),
        ModuleInfo::from_id(app_1::MOCK_APP_ID, V1.into()).unwrap(),
    ])?;
    assert_eq!(
        simulate_response,
        SimulateInstallModulesResponse {
            total_required_funds: vec![coin(10, "coin1"), coin(5, "coin2")],
            monetization_funds: vec![
                (adapter_1::MOCK_ADAPTER_ID.to_string(), coin(5, "coin1")),
                (adapter_2::MOCK_ADAPTER_ID.to_string(), coin(5, "coin1")),
                (app_1::MOCK_APP_ID.to_string(), coin(5, "coin2"))
            ],
            initialization_funds: vec![],
        }
    );

    let account = factory
        .create_new_account(
            AccountDetails {
                name: String::from("second_account"),
                description: None,
                link: None,
                namespace: None,
                base_asset: None,
                install_modules: vec![
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            adapter_1::MOCK_ADAPTER_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        None,
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            adapter_2::MOCK_ADAPTER_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        None,
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            app_1::MOCK_APP_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        Some(to_binary(&app::InstantiateMsg {
                            module: MockInitMsg,
                            base: BaseInstantiateMsg {
                                ans_host_address: deployment.ans_host.addr_str()?,
                                version_control_address: deployment.version_control.addr_str()?,
                            },
                        })?),
                    ),
                ],
            },
            GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            // we attach 5 extra coin2, rest should go to proxy
            Some(&[coin(10, "coin1"), coin(10, "coin2")]),
        )
        .unwrap();
    let balances = chain.query_all_balances(&account.proxy.address()?)?;
    assert_eq!(balances, vec![coin(5, "coin2")]);
    // Make sure all installed
    let account_module_versions = account.manager.module_versions(vec![
        String::from(adapter_1::MOCK_ADAPTER_ID),
        String::from(adapter_2::MOCK_ADAPTER_ID),
        String::from(app_1::MOCK_APP_ID),
    ])?;
    assert_eq!(
        account_module_versions,
        ModuleVersionsResponse {
            versions: vec![
                ContractVersion {
                    contract: String::from(adapter_1::MOCK_ADAPTER_ID),
                    version: String::from(V1)
                },
                ContractVersion {
                    contract: String::from(adapter_2::MOCK_ADAPTER_ID),
                    version: String::from(V1)
                },
                ContractVersion {
                    contract: String::from(app_1::MOCK_APP_ID),
                    version: String::from(V1)
                }
            ]
        }
    );
    Ok(())
}

#[test]
fn create_account_with_installed_module_and_monetization_should_fail() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    // Adding coins to fill monetization
    chain.add_balance(&sender, vec![coin(10, "coin1"), coin(10, "coin2")])?;
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let factory = &deployment.account_factory;

    let _deployer_acc = factory.create_new_account(
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            base_asset: None,
            install_modules: vec![],
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        None,
    )?;
    deploy_modules(&chain);
    // Add monetization
    deployment.version_control.update_module_configuration(
        "mock-adapter1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin1")))),
            instantiation_funds: None,
        },
    )?;
    deployment.version_control.update_module_configuration(
        "mock-adapter2".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin1")))),
            instantiation_funds: None,
        },
    )?;
    deployment.version_control.update_module_configuration(
        "mock-app1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin2")))),
            instantiation_funds: None,
        },
    )?;

    // Check how much we need
    let simulate_response = deployment.module_factory.simulate_install_modules(vec![
        ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
        ModuleInfo::from_id(adapter_2::MOCK_ADAPTER_ID, V1.into()).unwrap(),
        ModuleInfo::from_id(app_1::MOCK_APP_ID, V1.into()).unwrap(),
    ])?;
    assert_eq!(
        simulate_response.total_required_funds,
        vec![coin(10, "coin1"), coin(5, "coin2")]
    );

    let result = factory.create_new_account(
        AccountDetails {
            name: String::from("second_account"),
            description: None,
            link: None,
            namespace: None,
            base_asset: None,
            install_modules: vec![
                ModuleInstallConfig::new(
                    ModuleInfo::from_id(
                        adapter_1::MOCK_ADAPTER_ID,
                        ModuleVersion::Version(V1.to_owned()),
                    )?,
                    None,
                ),
                ModuleInstallConfig::new(
                    ModuleInfo::from_id(
                        adapter_2::MOCK_ADAPTER_ID,
                        ModuleVersion::Version(V1.to_owned()),
                    )?,
                    None,
                ),
                ModuleInstallConfig::new(
                    ModuleInfo::from_id(app_1::MOCK_APP_ID, ModuleVersion::Version(V1.to_owned()))?,
                    Some(to_binary(&app::InstantiateMsg {
                        module: MockInitMsg,
                        base: BaseInstantiateMsg {
                            ans_host_address: deployment.ans_host.addr_str()?,
                            version_control_address: deployment.version_control.addr_str()?,
                        },
                    })?),
                ),
            ],
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        // we attach 1 less coin1
        Some(&[coin(9, "coin1"), coin(10, "coin2")]),
    );
    // Mock doesn't implement debug so we can't .unwrap_err, LOL
    let Err(AbstractInterfaceError::Orch(e)) = result else {
        panic!()
    };
    assert!(e.root().to_string().contains(&format!(
        "Expected {:?}, sent {:?}",
        simulate_response.total_required_funds,
        vec![coin(9, "coin1"), coin(10, "coin2")]
    )));

    Ok(())
}

#[test]
fn create_account_with_installed_module_and_init_funds() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    // Adding coins to fill monetization
    chain.add_balance(&sender, vec![coin(15, "coin1"), coin(10, "coin2")])?;
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let factory = &deployment.account_factory;

    let _deployer_acc = factory.create_new_account(
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            base_asset: None,
            install_modules: vec![],
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        None,
    )?;
    deploy_modules(&chain);

    let standalone_contract = Box::new(ContractWrapper::new(
        standalone_no_cw2::mock_execute,
        standalone_no_cw2::mock_instantiate,
        standalone_no_cw2::mock_query,
    ));
    let standalone_id = chain.app.borrow_mut().store_code(standalone_contract);

    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("tester")?,
            name: "standalone".to_owned(),
            version: ModuleVersion::Version(V1.to_owned()),
        },
        ModuleReference::Standalone(standalone_id),
    )])?;

    // Add init_funds
    deployment.version_control.update_module_configuration(
        "mock-app1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: None,
            instantiation_funds: Some(vec![coin(3, "coin1"), coin(5, "coin2")]),
        },
    )?;
    deployment.version_control.update_module_configuration(
        "standalone".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: None,
            instantiation_funds: Some(vec![coin(6, "coin1")]),
        },
    )?;

    // Check how much we need
    let simulate_response = deployment.module_factory.simulate_install_modules(vec![
        ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
        ModuleInfo::from_id(adapter_2::MOCK_ADAPTER_ID, V1.into()).unwrap(),
        ModuleInfo::from_id(app_1::MOCK_APP_ID, V1.into()).unwrap(),
        ModuleInfo {
            namespace: Namespace::new("tester")?,
            name: "standalone".to_owned(),
            version: V1.into(),
        },
    ])?;
    assert_eq!(
        simulate_response,
        SimulateInstallModulesResponse {
            total_required_funds: vec![coin(9, "coin1"), coin(5, "coin2")],
            monetization_funds: vec![],
            initialization_funds: vec![
                (
                    app_1::MOCK_APP_ID.to_string(),
                    vec![coin(3, "coin1"), coin(5, "coin2")]
                ),
                ("tester:standalone".to_string(), vec![coin(6, "coin1")])
            ],
        }
    );

    let account = factory
        .create_new_account(
            AccountDetails {
                name: String::from("second_account"),
                description: None,
                link: None,
                namespace: None,
                base_asset: None,
                install_modules: vec![
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            adapter_1::MOCK_ADAPTER_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        None,
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            adapter_2::MOCK_ADAPTER_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        None,
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            app_1::MOCK_APP_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        Some(to_binary(&app::InstantiateMsg {
                            module: MockInitMsg,
                            base: BaseInstantiateMsg {
                                ans_host_address: deployment.ans_host.addr_str()?,
                                version_control_address: deployment.version_control.addr_str()?,
                            },
                        })?),
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo {
                            namespace: Namespace::new("tester")?,
                            name: "standalone".to_owned(),
                            version: V1.into(),
                        },
                        Some(to_binary(&MockInitMsg)?),
                    ),
                ],
            },
            GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            // we attach 1 extra coin1 and 5 extra coin2, rest should go to proxy
            Some(&[coin(10, "coin1"), coin(10, "coin2")]),
        )
        .unwrap();
    let balances = chain.query_all_balances(&account.proxy.address()?)?;
    assert_eq!(balances, vec![coin(1, "coin1"), coin(5, "coin2")]);
    // Make sure all installed
    Ok(())
}

#[test]
fn create_account_with_installed_module_monetization_and_init_funds() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    // Adding coins to fill monetization
    chain.add_balance(&sender, vec![coin(18, "coin1"), coin(20, "coin2")])?;
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let factory = &deployment.account_factory;

    let _deployer_acc = factory.create_new_account(
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            base_asset: None,
            install_modules: vec![],
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        None,
    )?;
    deploy_modules(&chain);

    let standalone_contract = Box::new(ContractWrapper::new(
        standalone_cw2::mock_execute,
        standalone_cw2::mock_instantiate,
        standalone_cw2::mock_query,
    ));
    let standalone_id = chain.app.borrow_mut().store_code(standalone_contract);

    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("tester")?,
            name: "standalone".to_owned(),
            version: ModuleVersion::Version(V1.to_owned()),
        },
        ModuleReference::Standalone(standalone_id),
    )])?;

    // Add init_funds
    deployment.version_control.update_module_configuration(
        "mock-app1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(10, "coin2")))),
            instantiation_funds: Some(vec![coin(3, "coin1"), coin(5, "coin2")]),
        },
    )?;
    deployment.version_control.update_module_configuration(
        "standalone".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(8, "coin1")))),
            instantiation_funds: Some(vec![coin(6, "coin1")]),
        },
    )?;

    // Check how much we need
    let simulate_response = deployment.module_factory.simulate_install_modules(vec![
        ModuleInfo::from_id(adapter_1::MOCK_ADAPTER_ID, V1.into()).unwrap(),
        ModuleInfo::from_id(adapter_2::MOCK_ADAPTER_ID, V1.into()).unwrap(),
        ModuleInfo::from_id(app_1::MOCK_APP_ID, V1.into()).unwrap(),
        ModuleInfo {
            namespace: Namespace::new("tester")?,
            name: "standalone".to_owned(),
            version: V1.into(),
        },
    ])?;
    assert_eq!(
        simulate_response,
        SimulateInstallModulesResponse {
            total_required_funds: vec![coin(17, "coin1"), coin(15, "coin2")],
            monetization_funds: vec![
                (app_1::MOCK_APP_ID.to_string(), coin(10, "coin2")),
                ("tester:standalone".to_string(), coin(8, "coin1"))
            ],
            initialization_funds: vec![
                (
                    app_1::MOCK_APP_ID.to_string(),
                    vec![coin(3, "coin1"), coin(5, "coin2")]
                ),
                ("tester:standalone".to_string(), vec![coin(6, "coin1")]),
            ],
        }
    );

    let account = factory
        .create_new_account(
            AccountDetails {
                name: String::from("second_account"),
                description: None,
                link: None,
                namespace: None,
                base_asset: None,
                install_modules: vec![
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            adapter_1::MOCK_ADAPTER_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        None,
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            adapter_2::MOCK_ADAPTER_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        None,
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo::from_id(
                            app_1::MOCK_APP_ID,
                            ModuleVersion::Version(V1.to_owned()),
                        )?,
                        Some(to_binary(&app::InstantiateMsg {
                            module: MockInitMsg,
                            base: BaseInstantiateMsg {
                                ans_host_address: deployment.ans_host.addr_str()?,
                                version_control_address: deployment.version_control.addr_str()?,
                            },
                        })?),
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo {
                            namespace: Namespace::new("tester")?,
                            name: "standalone".to_owned(),
                            version: V1.into(),
                        },
                        Some(to_binary(&MockInitMsg)?),
                    ),
                ],
            },
            GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            // we attach 1 extra coin1 and 5 extra coin2, rest should go to proxy
            Some(&[coin(18, "coin1"), coin(20, "coin2")]),
        )
        .unwrap();
    let balances = chain.query_all_balances(&account.proxy.address()?)?;
    assert_eq!(balances, vec![coin(1, "coin1"), coin(5, "coin2")]);
    // Make sure all installed
    Ok(())
}

// See gen_app_mock for more details
#[test]
fn install_app_with_proxy_action() -> AResult {
    let sender = Addr::unchecked(common::OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy } = &account;
    abstr
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // install adapter 1
    let adapter1 = install_module_version(manager, &abstr, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // install adapter 2
    let adapter2 = install_module_version(manager, &abstr, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // Add balance to proxy so
    // app will transfer funds to test addr during instantiation
    chain.add_balance(&proxy.address()?, coins(123456, "TEST"))?;
    let app1 = install_module_version(manager, &abstr, app_1::MOCK_APP_ID, V1)?;

    let test_addr_balance = chain.query_balance(&Addr::unchecked("test_addr"), "TEST")?;
    assert_eq!(test_addr_balance, Uint128::new(123456));

    account.expect_modules(vec![adapter1, adapter2, app1])?;
    Ok(())
}

// TODO:
// - adapter-adapter dependencies
// - app-adapter dependencies
// - app-app dependencies
