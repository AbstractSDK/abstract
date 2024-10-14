use abstract_account::error::AccountError;
use abstract_app::mock::{MockInitMsg, MockMigrateMsg};
use abstract_integration_tests::{create_default_account, mock_modules::*, AResult, *};
use abstract_interface::{
    Abstract, AbstractInterfaceError, AccountDetails, AccountExecFns as _, AccountI,
    AccountQueryFns, MFactoryQueryFns, RegistryExecFns,
};
use abstract_std::{
    account::{ModuleInstallConfig, ModuleVersionsResponse},
    app, ibc_client,
    module_factory::SimulateInstallModulesResponse,
    objects::{
        fee::FixedFee,
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion, Monetization},
        module_reference::ModuleReference,
        namespace::Namespace,
        AccountId,
    },
    registry::UpdateModule,
    AbstractError, IBC_CLIENT,
};
use abstract_testing::prelude::*;
use assertor::*;
use cosmwasm_std::coin;
use cw2::ContractVersion;
use cw_orch::prelude::*;

#[test]
fn install_app_successful() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &abstr)?;

    abstr
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // dependency for mock_adapter1 not met
    let res = install_module_version(&account, app_1::MOCK_APP_ID, V1);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string()).contains(
        // Error from macro
        "no address",
    );

    // install adapter 1
    let adapter1 = install_module_version(&account, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // second dependency still not met
    let res = install_module_version(&account, app_1::MOCK_APP_ID, V1);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string()).contains(
        "module tester:mock-adapter2 is a dependency of tester:mock-app1 and is not installed.",
    );

    // install adapter 2
    let adapter2 = install_module_version(&account, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(&account, app_1::MOCK_APP_ID, V1)?;

    account.expect_modules(vec![adapter1, adapter2, app1])?;
    Ok(())
}

#[test]
fn install_app_versions_not_met() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &abstr)?;

    abstr
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // install adapter 2
    let _adapter2 = install_module_version(&account, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // successfully install app 1
    let _app1 = install_module_version(&account, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // attempt to install app with version 2

    let res = install_module_version(&account, app_1::MOCK_APP_ID, V2);
    assert_that!(&res).is_err();
    assert_that!(res.unwrap_err().root_cause().to_string())
        .contains("Module tester:mock-adapter1 with version 1.0.0 does not fit requirement ^2.0.0");
    Ok(())
}

#[test]
fn upgrade_app() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on_mock(chain.clone())?;
    let account = create_default_account(&sender, &abstr)?;

    abstr
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // install adapter 1
    let adapter1 = install_module_version(&account, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // install adapter 2
    let adapter2 = install_module_version(&account, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(&account, app_1::MOCK_APP_ID, V1)?;
    account.expect_modules(vec![adapter1, adapter2, app1])?;

    // attempt upgrade app 1 to version 2
    let res = account.upgrade_module(
        app_1::MOCK_APP_ID,
        &app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: MockMigrateMsg,
        },
    );
    // fails because adapter 1 is not version 2
    assert_that!(res.unwrap_err().root().to_string()).contains(
        AccountError::VersionRequirementNotMet {
            module_id: adapter_1::MOCK_ADAPTER_ID.into(),
            version: V1.into(),
            comp: "^2.0.0".into(),
            post_migration: true,
        }
        .to_string(),
    );

    // upgrade adapter 1 to version 2
    let res = account.upgrade_module(
        adapter_1::MOCK_ADAPTER_ID,
        &app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: Empty {},
        },
    );
    // fails because app v1 is not version 2 and depends on adapter 1 being version 1.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        AccountError::VersionRequirementNotMet {
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
    let res = account.upgrade(vec![
        (
            ModuleInfo::from_id(app_1::MOCK_APP_ID, ModuleVersion::Version(V1.to_string()))?,
            Some(to_json_binary(&app::MigrateMsg {
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
        AccountError::Abstract(AbstractError::CannotDowngradeContract {
            contract: app_1::MOCK_APP_ID.into(),
            from: V1.parse().unwrap(),
            to: V1.parse().unwrap(),
        })
        .to_string(),
    );

    // attempt to upgrade app 1 to version 2 while not updating other modules
    let res = account.upgrade(vec![(
        ModuleInfo::from_id(app_1::MOCK_APP_ID, ModuleVersion::Version(V2.to_string()))?,
        Some(to_json_binary(&app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: MockMigrateMsg,
        })?),
    )]);

    // fails because app v1 is depends on adapter 1 being version 2.
    assert_that!(res.unwrap_err().root().to_string()).contains(
        AccountError::VersionRequirementNotMet {
            module_id: adapter_1::MOCK_ADAPTER_ID.into(),
            version: V1.into(),
            comp: "^2.0.0".into(),
            post_migration: true,
        }
        .to_string(),
    );

    // attempt to upgrade app 1 to identical version while updating other modules
    let res = account.upgrade(vec![
        (
            ModuleInfo::from_id(app_1::MOCK_APP_ID, ModuleVersion::Version(V2.to_string()))?,
            Some(to_json_binary(&app::MigrateMsg {
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
        AccountError::VersionRequirementNotMet {
            module_id: adapter_1::MOCK_ADAPTER_ID.into(),
            version: V1.into(),
            comp: "^2.0.0".into(),
            post_migration: true,
        }
        .to_string(),
    );

    // successfully upgrade all the modules
    account.upgrade(vec![
        (
            ModuleInfo::from_id_latest(app_1::MOCK_APP_ID)?,
            Some(to_json_binary(&app::MigrateMsg {
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
    let chain = MockBech32::new("mock");
    Abstract::deploy_on_mock(chain.clone())?;
    abstract_integration_tests::account::uninstall_modules(chain)
}

#[test]
fn update_adapter_with_authorized_addrs() -> AResult {
    let chain = MockBech32::new("mock");
    Abstract::deploy_on_mock(chain.clone())?;
    abstract_integration_tests::account::update_adapter_with_authorized_addrs(
        chain.clone(),
        chain.addr_make("authorizee"),
    )
}

/*#[test]
fn upgrade_account_last() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let abstr = Abstract::deploy_on(chain.clone(), mock_bech32_sender(&chain))?;
    let account = create_default_account(&sender,&abstr)?;
    let AccountI { account, account: _ } = &account;

    abstr
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, vec![TEST_NAMESPACE.to_string()])?;
    deploy_modules(&chain);

    // install adapter 1
    let adapter1 = install_module_version(&account, &abstr, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // install adapter 2
    let adapter2 = install_module_version(&account, &abstr, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // successfully install app 1
    let app1 = install_module_version(&account, &abstr, app_1::MOCK_APP_ID, V1)?;
    account.expect_modules(vec![adapter1, adapter2, app1])?;

    // Upgrade all modules, including the account module, but ensure the account is upgraded last
    let res = account.upgrade(vec![
        (
            ModuleInfo::from_id_latest(app_1::MOCK_APP_ID)?,
            Some(to_json_binary(&app::MigrateMsg {
                base: app::BaseMigrateMsg {},
                module: MockMigrateMsg,
            })?),
        ),
        (
            ModuleInfo::from_id_latest("abstract:account")?,
            Some(to_json_binary(&account::MigrateMsg {})?),
        ),
        (ModuleInfo::from_id_latest(adapter_1::MOCK_ADAPTER_ID)?, None),
        (ModuleInfo::from_id_latest(adapter_2::MOCK_ADAPTER_ID)?, None),
    ])?;

    // get the events
    let mut events: Vec<Event> = res.events;
    events.pop();
    let migrate_event = events.pop().unwrap();

    // the 2nd last event will be the account execution
    assert_that!(migrate_event.attributes).has_length(3);
    let mut attributes = migrate_event.attributes;
    // check that the action was migrate
    assert_that!(attributes.pop())
        .is_some()
        .is_equal_to(Attribute::from(("action", "migrate")));

    // and that it was the account
    assert_that!(attributes.pop())
        .is_some()
        .is_equal_to(Attribute::from(("contract", "abstract:account")));

    Ok(())
}*/

#[test]
fn no_duplicate_migrations() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let abstr = Abstract::deploy_on_mock(chain.clone())?;

    let account = create_default_account(&sender, &abstr)?;

    abstr
        .registry
        .claim_namespace(TEST_ACCOUNT_ID, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // Install adapter 1
    let adapter1 = install_module_version(&account, adapter_1::MOCK_ADAPTER_ID, V1)?;

    account.expect_modules(vec![adapter1])?;

    // Upgrade all modules, including the account module
    let res = account.upgrade(vec![
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
        AccountError::DuplicateModuleMigration {
            module_id: adapter_1::MOCK_ADAPTER_ID.to_string(),
        }
        .to_string(),
    );

    Ok(())
}

#[test]
fn create_account_with_installed_module() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    let _deployer_acc = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            install_modules: vec![],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    )?;
    deploy_modules(&chain);

    let account = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("second_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
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
                    Some(to_json_binary(&MockInitMsg {})?),
                ),
            ],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    )
    .unwrap();

    // Make sure all installed
    let account_module_versions = account.module_versions(vec![
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
    let chain = MockBech32::new("mock");
    Abstract::deploy_on_mock(chain.clone())?;
    abstract_integration_tests::account::create_sub_account_with_modules_installed(chain)
}

#[test]
fn create_account_with_installed_module_and_monetization() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    // Adding coins to fill monetization
    chain.add_balance(&sender, vec![coin(10, "coin1"), coin(10, "coin2")])?;
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    let _deployer_acc = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            install_modules: vec![],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    )?;
    deploy_modules(&chain);
    // Add monetization
    deployment.registry.update_module_configuration(
        "mock-adapter1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin1")))),
            instantiation_funds: None,
        },
    )?;
    deployment.registry.update_module_configuration(
        "mock-adapter2".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin1")))),
            instantiation_funds: None,
        },
    )?;
    deployment.registry.update_module_configuration(
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

    let account = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("second_account"),
            description: None,
            link: None,
            namespace: None,
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
                    Some(to_json_binary(&MockInitMsg {})?),
                ),
            ],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        // we attach 5 extra coin2, rest should go to account
        &[coin(10, "coin1"), coin(10, "coin2")],
    )
    .unwrap();
    let balances = chain.query_all_balances(&account.address()?)?;
    assert_eq!(balances, vec![coin(5, "coin2")]);
    // Make sure all installed
    let account_module_versions = account.module_versions(vec![
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
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    // Adding coins to fill monetization
    chain.add_balance(&sender, vec![coin(10, "coin1"), coin(10, "coin2")])?;
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    let _deployer_acc = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            install_modules: vec![],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    )?;
    deploy_modules(&chain);
    // Add monetization
    deployment.registry.update_module_configuration(
        "mock-adapter1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin1")))),
            instantiation_funds: None,
        },
    )?;
    deployment.registry.update_module_configuration(
        "mock-adapter2".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(5, "coin1")))),
            instantiation_funds: None,
        },
    )?;
    deployment.registry.update_module_configuration(
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

    let result = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("second_account"),
            description: None,
            link: None,
            namespace: None,
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
                    Some(to_json_binary(&MockInitMsg {})?),
                ),
            ],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        // we attach 1 less coin1
        &[coin(9, "coin1"), coin(10, "coin2")],
    );
    // Mock doesn't implement debug so we can't .unwrap_err, LOL
    let Err(AbstractInterfaceError::Orch(e)) = result else {
        panic!()
    };
    eprintln!("{:?}", e);
    assert!(e
        .root()
        .to_string()
        .contains(&"Expected 10coin1,5coin2, sent".to_string()));

    Ok(())
}

#[test]
fn create_account_with_installed_module_and_init_funds() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    // Adding coins to fill monetization
    chain.add_balance(&sender, vec![coin(15, "coin1"), coin(10, "coin2")])?;
    let deployment = Abstract::deploy_on_mock(chain.clone())?;

    let _deployer_acc = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(String::from(TEST_NAMESPACE)),
            install_modules: vec![],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    )?;
    deploy_modules(&chain);

    let standalone_contract = Box::new(ContractWrapper::new(
        standalone_no_cw2::mock_execute,
        standalone_no_cw2::mock_instantiate,
        standalone_no_cw2::mock_query,
    ));
    let standalone_id = chain.app.borrow_mut().store_code(standalone_contract);

    deployment.registry.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("tester")?,
            name: "standalone".to_owned(),
            version: ModuleVersion::Version(V1.to_owned()),
        },
        ModuleReference::Standalone(standalone_id),
    )])?;

    // Add init_funds
    deployment.registry.update_module_configuration(
        "mock-app1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: None,
            instantiation_funds: Some(vec![coin(3, "coin1"), coin(5, "coin2")]),
        },
    )?;
    deployment.registry.update_module_configuration(
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

    let account = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("second_account"),
            description: None,
            link: None,
            namespace: None,
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
                    Some(to_json_binary(&MockInitMsg {})?),
                ),
                ModuleInstallConfig::new(
                    ModuleInfo {
                        namespace: Namespace::new("tester")?,
                        name: "standalone".to_owned(),
                        version: V1.into(),
                    },
                    Some(to_json_binary(&MockInitMsg {})?),
                ),
            ],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        // we attach 1 extra coin1 and 5 extra coin2, rest should go to account
        &[coin(10, "coin1"), coin(10, "coin2")],
    )
    .unwrap();
    let balances = chain.query_all_balances(&account.address()?)?;
    assert_eq!(balances, vec![coin(1, "coin1"), coin(5, "coin2")]);
    // Make sure all installed
    Ok(())
}

#[test]
fn create_account_with_installed_module_monetization_and_init_funds() -> AResult {
    let chain = MockBech32::new("mock");
    Abstract::deploy_on_mock(chain.clone())?;
    abstract_integration_tests::account::create_account_with_installed_module_monetization_and_init_funds(chain, ("coin1", "coin2"))
}

// See gen_app_mock for more details
#[test]
fn install_app_with_account_action() -> AResult {
    let chain = MockBech32::new("mock");
    Abstract::deploy_on_mock(chain.clone())?;
    abstract_integration_tests::account::install_app_with_account_action(chain)
}

#[test]
fn native_not_migratable() -> AResult {
    let mut chain = MockBech32::new("mock");
    let admin = Abstract::mock_admin(&chain);
    chain.set_sender(admin.clone());
    let abstr = Abstract::deploy_on(chain.clone(), admin.clone())?;
    let abstr_account = AccountI::load_from(&abstr, AccountId::local(0))?;
    abstr_account.install_module::<ibc_client::InstantiateMsg>(IBC_CLIENT, None, &[])?;

    let latest_ibc_client = ModuleInfo::from_id_latest(IBC_CLIENT).unwrap();

    let err: AccountError = abstr_account
        .upgrade(vec![(latest_ibc_client.clone(), None)])
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(err, AccountError::NotUpgradeable(latest_ibc_client));
    Ok(())
}

// TODO:
// - adapter-adapter dependencies
// - app-adapter dependencies
// - app-app dependencies
