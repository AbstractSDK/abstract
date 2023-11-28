use crate::mock_modules::app_1::*;
use crate::mock_modules::standalone_cw2;
use crate::mock_modules::*;
use crate::AResult;
use abstract_app::gen_app_mock;
use abstract_app::mock::MockInitMsg;
use abstract_core::manager::ModuleInstallConfig;
use abstract_core::manager::ModuleVersionsResponse;
use abstract_core::module_factory::SimulateInstallModulesResponse;
use abstract_core::objects::account::TEST_ACCOUNT_ID;
use abstract_core::objects::fee::FixedFee;
use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_core::objects::module::ModuleInfo;
use abstract_core::objects::module::ModuleVersion;
use abstract_core::objects::module::Monetization;
use abstract_core::objects::module_reference::ModuleReference;
use abstract_core::objects::namespace::Namespace;
use abstract_core::objects::AccountId;
use abstract_core::version_control::UpdateModule;
use abstract_interface::*;
use abstract_testing::prelude::*;
use cosmwasm_std::coin;
use cosmwasm_std::Addr;
use cw2::ContractVersion;
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use speculoos::prelude::*;

/// Test installing an app on an account
pub fn account_install_app<T: CwEnv>(chain: T, sender: Addr) -> AResult {
    let deployment = Abstract::load_from(chain.clone())?;
    let account = crate::create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(TEST_ACCOUNT_ID, "tester".to_owned())?;

    let app = BootMockApp1V1::new_test(chain.clone());
    BootMockApp1V1::deploy(&app, V1.parse().unwrap(), DeployStrategy::Try)?;
    let app_addr = account.install_app(&app, &MockInitMsg, None)?;
    let module_addr = account
        .manager
        .module_info(app_1::MOCK_APP_ID)?
        .unwrap()
        .address;
    assert_that!(app_addr).is_equal_to(module_addr);
    Ok(())
}

/// Test installing an app on an account
pub fn create_sub_account_with_modules_installed<T: CwEnv>(chain: T, sender: Addr) -> AResult {
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let factory = &deployment.account_factory;

    let deployer_acc = factory.create_new_account(
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
    crate::mock_modules::deploy_modules(&chain);

    deployer_acc.manager.create_sub_account(
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
                Some(to_json_binary(&MockInitMsg)?),
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

pub fn create_account_with_installed_module_monetization_and_init_funds<T: CwEnv>(
    chain: T,
    sender: Addr,
    payment_denoms: (&str, &str),

) -> AResult {
    // Adding coins to fill monetization
    // chain.add_balance(&sender, vec![coin(18, "coin1"), coin(20, "coin2")])?;

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

    let standalone = standalone_cw2::StandaloneCw2::new_test(chain);
    standalone.upload()?;

    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("tester")?,
            name: "standalone".to_owned(),
            version: ModuleVersion::Version(V1.to_owned()),
        },
        ModuleReference::Standalone(standalone.code_id()?),
    )])?;

    // Add init_funds
    deployment.version_control.update_module_configuration(
        "mock-app1".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(
                10,
                payment_denoms.1,
            )))),
            instantiation_funds: Some(vec![coin(3, payment_denoms.0), coin(5, payment_denoms.1)]),
        },
    )?;
    deployment.version_control.update_module_configuration(
        "standalone".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(
                8,
                payment_denoms.0,
            )))),
            instantiation_funds: Some(vec![coin(6, payment_denoms.0)]),
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
            total_required_funds: vec![coin(17, payment_denoms.0), coin(15, payment_denoms.1)],
            monetization_funds: vec![
                (app_1::MOCK_APP_ID.to_string(), coin(10, payment_denoms.1)),
                ("tester:standalone".to_string(), coin(8, payment_denoms.0))
            ],
            initialization_funds: vec![
                (
                    app_1::MOCK_APP_ID.to_string(),
                    vec![coin(3, payment_denoms.0), coin(5, payment_denoms.1)]
                ),
                (
                    "tester:standalone".to_string(),
                    vec![coin(6, payment_denoms.0)]
                ),
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
                        Some(to_json_binary(&MockInitMsg)?),
                    ),
                    ModuleInstallConfig::new(
                        ModuleInfo {
                            namespace: Namespace::new("tester")?,
                            name: "standalone".to_owned(),
                            version: V1.into(),
                        },
                        Some(to_json_binary(&MockInitMsg)?),
                    ),
                ],
            },
            GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            // we attach 1 extra coin1 and 5 extra coin2, rest should go to proxy
            Some(&[coin(18, payment_denoms.0), coin(20, payment_denoms.1)]),
        )
        .unwrap();
    let balances = chain.query_all_balances(&account.proxy.address()?)?;
    assert_eq!(
        balances,
        vec![coin(1, payment_denoms.0), coin(5, payment_denoms.1)]
    );
    // Make sure all installed
    Ok(())
}
