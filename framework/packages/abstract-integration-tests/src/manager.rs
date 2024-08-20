use abstract_adapter::mock::{MockExecMsg, MockReceiveMsg};
use abstract_app::mock::MockInitMsg;
use abstract_interface::*;
use abstract_manager::error::ManagerError;
use abstract_std::{
    adapter::{AdapterBaseMsg, AdapterRequestMsg},
    app,
    manager::{ModuleInstallConfig, ModuleVersionsResponse},
    module_factory::SimulateInstallModulesResponse,
    objects::{
        fee::FixedFee,
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion, Monetization},
        module_reference::ModuleReference,
        namespace::Namespace,
        ownership, AccountId,
    },
    version_control::UpdateModule,
    PROXY,
};
use abstract_testing::prelude::*;
use cosmwasm_std::{coin, coins, wasm_execute, Uint128};
use cw2::ContractVersion;
use cw_orch::{environment::MutCwEnv, prelude::*};
use speculoos::prelude::*;

use crate::{
    add_mock_adapter_install_fee, create_default_account, init_mock_adapter, install_adapter,
    install_adapter_with_funds, install_module_version, mock_modules::*, AResult,
};

pub mod mock_app {
    use abstract_app::gen_app_mock;

    pub const APP_ID: &str = "tester:app";
    pub const APP_VERSION: &str = "1.0.0";
    gen_app_mock!(MockApp, APP_ID, APP_VERSION, &[]);
}
use mock_app::*;

/// Test installing an app on an account
pub fn account_install_app<T: CwEnv>(chain: T) -> AResult {
    let deployment = Abstract::load_from(chain.clone())?;
    let account = crate::create_default_account(&deployment.account_factory)?;

    deployment
        .version_control
        .claim_namespace(account.id()?, "tester".to_owned())?;

    let app = MockApp::new_test(chain.clone());
    MockApp::deploy(&app, APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
    let app_addr = account.install_app(&app, &MockInitMsg {}, &[])?;
    let module_addr = account.manager.module_info(APP_ID)?.unwrap().address;
    assert_that!(app_addr).is_equal_to(module_addr);
    Ok(())
}

/// Test installing an app on an account
pub fn create_sub_account_with_modules_installed<T: CwEnv>(chain: T) -> AResult {
    let deployment = Abstract::load_from(chain.clone())?;
    let sender = chain.sender_addr();
    let factory = &deployment.account_factory;

    let deployer_acc = factory.create_new_account(
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
                Some(to_json_binary(&MockInitMsg {})?),
            ),
        ],
        String::from("sub_account"),
        None,
        Some(String::from("account_description")),
        None,
        None,
        &[],
    )?;

    let sub_account_id = deployer_acc
        .manager
        .sub_account_ids(None, None)?
        .sub_accounts[0];
    let sub_account = AbstractAccount::new(&deployment, AccountId::local(sub_account_id));

    // Make sure all installed
    let account_module_versions = sub_account.manager.module_versions(vec![
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

pub fn create_account_with_installed_module_monetization_and_init_funds<T: MutCwEnv>(
    mut chain: T,
    (coin1, coin2): (&str, &str),
) -> AResult {
    let sender = chain.sender_addr();
    // Adding coins to fill monetization
    chain
        .add_balance(&sender, vec![coin(18, coin1), coin(20, coin2)])
        .unwrap();
    let deployment = Abstract::load_from(chain.clone())?;
    let factory = &deployment.account_factory;

    let _deployer_acc = factory.create_new_account(
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

    let standalone = standalone_cw2::StandaloneCw2::new_test(chain.clone());
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
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(10, coin2)))),
            instantiation_funds: Some(vec![coin(3, coin1), coin(5, coin2)]),
        },
    )?;
    deployment.version_control.update_module_configuration(
        "standalone".to_owned(),
        Namespace::new("tester").unwrap(),
        UpdateModule::Versioned {
            version: V1.to_owned(),
            metadata: None,
            monetization: Some(Monetization::InstallFee(FixedFee::new(&coin(8, coin1)))),
            instantiation_funds: Some(vec![coin(6, coin1)]),
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
            total_required_funds: vec![coin(17, coin1), coin(15, coin2)],
            monetization_funds: vec![
                (app_1::MOCK_APP_ID.to_string(), coin(10, coin2)),
                ("tester:standalone".to_string(), coin(8, coin1))
            ],
            initialization_funds: vec![
                (
                    app_1::MOCK_APP_ID.to_string(),
                    vec![coin(3, coin1), coin(5, coin2)]
                ),
                ("tester:standalone".to_string(), vec![coin(6, coin1)]),
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
            // we attach 1 extra coin1 and 5 extra coin2, rest should go to proxy
            &[coin(18, coin1), coin(20, coin2)],
        )
        .unwrap();
    let balances = chain
        .bank_querier()
        .balance(&account.proxy.address()?, None)
        .unwrap();
    assert_eq!(balances, vec![coin(1, coin1), coin(5, coin2)]);
    Ok(())
}

pub fn install_app_with_proxy_action<T: MutCwEnv>(mut chain: T) -> AResult {
    let abstr = Abstract::load_from(chain.clone())?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy } = &account;
    abstr
        .version_control
        .claim_namespace(account.id()?, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // install adapter 1
    let adapter1 = install_module_version(manager, adapter_1::MOCK_ADAPTER_ID, V1)?;

    // install adapter 2
    let adapter2 = install_module_version(manager, adapter_2::MOCK_ADAPTER_ID, V1)?;

    // Add balance to proxy so
    // app will transfer funds to adapter1 addr during instantiation
    chain
        .add_balance(&proxy.address()?, coins(123456, "TEST"))
        .unwrap();
    let app1 = install_module_version(manager, app_1::MOCK_APP_ID, V1)?;

    let test_addr_balance = chain
        .bank_querier()
        .balance(&Addr::unchecked(&adapter1), Some("TEST".to_owned()))
        .unwrap();
    assert_eq!(test_addr_balance[0].amount, Uint128::new(123456));

    account.expect_modules(vec![adapter1, adapter2, app1])?;
    Ok(())
}

pub fn update_adapter_with_authorized_addrs<T: CwEnv>(chain: T, authorizee: Addr) -> AResult {
    let abstr = Abstract::load_from(chain.clone())?;
    let account = create_default_account(&abstr.account_factory)?;
    let AbstractAccount { manager, proxy } = &account;
    abstr
        .version_control
        .claim_namespace(account.id()?, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    // install adapter 1
    let adapter1 = install_module_version(manager, adapter_1::MOCK_ADAPTER_ID, V1)?;
    account.expect_modules(vec![adapter1.clone()])?;

    // register an authorized address on Adapter 1
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
    use abstract_std::manager::QueryMsgFns as _;

    let adapter_v2 = manager.module_addresses(vec![adapter_1::MOCK_ADAPTER_ID.into()])?;
    // assert that the address actually changed
    assert_that!(adapter_v2.modules[0].1).is_not_equal_to(Addr::unchecked(adapter1.clone()));

    let adapter = adapter_1::MockAdapterI1V2::new_test(chain);
    use abstract_std::adapter::BaseQueryMsgFns as _;
    let authorized = adapter.authorized_addresses(proxy.addr_str()?)?;
    assert_that!(authorized.addresses).contains(authorizee);

    // assert that authorized address was removed from old Adapter
    adapter.set_address(&Addr::unchecked(adapter1));
    let authorized = adapter.authorized_addresses(proxy.addr_str()?)?;
    assert_that!(authorized.addresses).is_empty();
    Ok(())
}

pub fn uninstall_modules<T: CwEnv>(chain: T) -> AResult {
    let deployment = Abstract::load_from(chain.clone())?;
    let account = create_default_account(&deployment.account_factory)?;
    let AbstractAccount { manager, proxy: _ } = &account;
    deployment
        .version_control
        .claim_namespace(account.id()?, TEST_NAMESPACE.to_string())?;
    deploy_modules(&chain);

    let adapter1 = install_module_version(manager, adapter_1::MOCK_ADAPTER_ID, V1)?;
    let adapter2 = install_module_version(manager, adapter_2::MOCK_ADAPTER_ID, V1)?;
    let app1 = install_module_version(manager, app_1::MOCK_APP_ID, V1)?;
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

pub fn installing_one_adapter_with_fee_should_succeed<T: MutCwEnv>(mut chain: T) -> AResult {
    let sender = chain.sender_addr();
    let deployment = Abstract::load_from(chain.clone())?;
    let account = create_default_account(&deployment.account_factory)?;
    chain.set_balance(&sender, coins(45, "ujunox")).unwrap();

    init_mock_adapter(chain.clone(), &deployment, None, account.id()?)?;
    add_mock_adapter_install_fee(
        &deployment,
        Monetization::InstallFee(FixedFee::new(&coin(45, "ujunox"))),
        None,
    )?;

    assert_that!(install_adapter_with_funds(
        &account.manager,
        TEST_MODULE_ID,
        &coins(45, "ujunox")
    ))
    .is_ok();

    Ok(())
}

pub fn with_response_data<T: MutCwEnv<Sender = Addr>>(mut chain: T) -> AResult {
    let deployment = Abstract::load_from(chain.clone())?;
    let account = create_default_account(&deployment.account_factory)?;

    let staking_adapter = init_mock_adapter(chain.clone(), &deployment, None, account.id()?)?;

    install_adapter(&account.manager, TEST_MODULE_ID)?;

    let manager_address = account.manager.address()?;
    staking_adapter.call_as(&manager_address).execute(
        &abstract_std::adapter::ExecuteMsg::<MockExecMsg, MockReceiveMsg>::Base(
            abstract_std::adapter::BaseExecuteMsg {
                proxy_address: None,
                msg: AdapterBaseMsg::UpdateAuthorizedAddresses {
                    to_add: vec![account.proxy.addr_str()?],
                    to_remove: vec![],
                },
            },
        ),
        &[],
    )?;

    chain
        .set_balance(
            &account.proxy.address()?,
            vec![Coin::new(100_000u128, TTOKEN)],
        )
        .unwrap();

    let adapter_addr = account
        .manager
        .module_info(TEST_MODULE_ID)?
        .expect("test module installed");
    // proxy should be final executor because of the reply
    let resp = account.manager.exec_on_module(
        cosmwasm_std::to_json_binary(&abstract_std::proxy::ExecuteMsg::ModuleActionWithData {
            // execute a message on the adapter, which sets some data in its response
            msg: wasm_execute(
                adapter_addr.address,
                &abstract_std::adapter::ExecuteMsg::<MockExecMsg, Empty>::Module(
                    AdapterRequestMsg {
                        proxy_address: Some(account.proxy.addr_str()?),
                        request: MockExecMsg {},
                    },
                ),
                vec![],
            )?
            .into(),
        })?,
        PROXY.to_string(),
        &[],
    )?;

    let response_data_attr_present = resp.event_attr_value("wasm-abstract", "response_data")?;
    assert_that!(response_data_attr_present).is_equal_to("true".to_string());
    Ok(())
}

pub fn account_move_ownership_to_sub_account<T: CwEnv<Sender = Addr>>(chain: T) -> AResult {
    let deployment = Abstract::load_from(chain)?;
    let account = create_default_account(&deployment.account_factory)?;

    account.manager.create_sub_account(
        vec![],
        "My subaccount".to_string(),
        None,
        None,
        None,
        None,
        &[],
    )?;
    let ids = account.manager.sub_account_ids(None, None)?;
    let sub_account_id = ids.sub_accounts[0];
    let sub_account = AbstractAccount::new(&deployment, AccountId::local(sub_account_id));
    let sub_manager_addr = sub_account.manager.address()?;
    let sub_proxy_addr = sub_account.proxy.address()?;

    let new_account = create_default_account(&deployment.account_factory)?;
    let new_governance = GovernanceDetails::SubAccount {
        manager: sub_manager_addr.to_string(),
        proxy: sub_proxy_addr.to_string(),
    };
    new_account
        .manager
        .update_ownership(ownership::GovAction::TransferOwnership {
            new_owner: new_governance.clone(),
            expiry: None,
        })?;
    let new_account_manager = new_account.manager.address()?;
    let new_account_id = new_account.id()?;

    let sub_account = AbstractAccount::new(&deployment, AccountId::local(sub_account_id));
    sub_account
        .proxy
        .call_as(&sub_manager_addr)
        .module_action(vec![wasm_execute(
            new_account_manager,
            &abstract_std::manager::ExecuteMsg::UpdateOwnership(
                ownership::GovAction::AcceptOwnership,
            ),
            vec![],
        )?
        .into()])?;

    // sub-accounts state updated
    let sub_ids = sub_account.manager.sub_account_ids(None, None)?;
    assert_eq!(sub_ids.sub_accounts, vec![new_account_id.seq()]);

    // owner of new_account updated
    let new_account = AbstractAccount::new(&deployment, AccountId::local(new_account_id.seq()));
    let owner = new_account.manager.ownership()?.owner;
    assert_eq!(new_governance, owner);

    Ok(())
}
