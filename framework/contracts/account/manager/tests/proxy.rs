use abstract_adapter::mock::{MockExecMsg, MockInitMsg};
use abstract_integration_tests::*;
use abstract_interface::*;
use abstract_manager::{contract::CONTRACT_VERSION, error::ManagerError};
use abstract_std::{
    manager::{InfoResponse, ManagerModuleInfo, ModuleInstallConfig, ModuleVersionsResponse},
    objects::{
        fee::FixedFee,
        gov_type::GovernanceDetails,
        module::{ModuleInfo, ModuleVersion, Monetization},
        module_reference::ModuleReference,
        namespace::Namespace,
        AccountId, ABSTRACT_ACCOUNT_ID,
    },
    version_control::{NamespaceResponse, UpdateModule},
    PROXY,
};
use abstract_testing::prelude::*;
use cosmwasm_std::{coin, CosmosMsg};
use cw_orch::prelude::*;
use speculoos::prelude::*;

#[test]
fn instantiate() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    let modules = account.manager.module_infos(None, None)?.module_infos;

    // assert proxy module
    assert_that!(&modules).has_length(1);
    assert_that(&modules[0]).is_equal_to(&ManagerModuleInfo {
        address: account.proxy.address()?,
        id: PROXY.to_string(),
        version: cw2::ContractVersion {
            contract: PROXY.into(),
            version: CONTRACT_VERSION.into(),
        },
    });

    // assert manager config
    assert_that!(account.manager.config()?).is_equal_to(abstract_std::manager::ConfigResponse {
        version_control_address: deployment.version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        account_id: TEST_ACCOUNT_ID,
        is_suspended: false,
    });
    take_storage_snapshot!(chain, "instantiate_proxy");
    Ok(())
}

/// ANCHOR: mock_integration_test
#[test]
fn exec_through_manager() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    // This testing environments allows you to use simple deploy contraptions:
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    // mint coins to proxy address
    chain.set_balance(&account.proxy.address()?, vec![Coin::new(100_000, TTOKEN)])?;

    // burn coins from proxy
    let proxy_balance = chain
        .bank_querier()
        .balance(account.proxy.address()?, None)?;

    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000, TTOKEN)]);

    let burn_amount = vec![Coin::new(10_000, TTOKEN)];

    account.manager.exec_on_module(
        cosmwasm_std::to_json_binary(&abstract_std::proxy::ExecuteMsg::ModuleAction {
            msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
                amount: burn_amount,
            })],
        })?,
        PROXY.to_string(),
        &[],
    )?;

    let proxy_balance = chain
        .bank_querier()
        .balance(account.proxy.address()?, None)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000 - 10_000, TTOKEN)]);
    take_storage_snapshot!(chain, "exec_through_manager");

    Ok(())
}
/// ANCHOR_END: mock_integration_test

#[test]
fn default_without_response_data() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;
    let _staking_adapter_one = init_mock_adapter(chain.clone(), &deployment, None, account.id()?)?;

    install_adapter(&account.manager, TEST_MODULE_ID)?;

    chain.set_balance(&account.proxy.address()?, vec![Coin::new(100_000, TTOKEN)])?;

    let resp = account.manager.execute_on_module(
        TEST_MODULE_ID,
        Into::<abstract_std::adapter::ExecuteMsg<MockExecMsg>>::into(MockExecMsg {}),
    )?;
    assert_that!(resp.data).is_none();
    take_storage_snapshot!(chain, "default_without_response_data");

    Ok(())
}

#[test]
fn with_response_data() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    Abstract::deploy_on(chain.clone(), sender.to_string())?;
    abstract_integration_tests::manager::with_response_data(chain.clone())?;
    take_storage_snapshot!(chain, "proxy_with_response_data");

    Ok(())
}

#[test]
fn install_standalone_modules() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AbstractAccount::new(&deployment, AccountId::local(0));

    let standalone1_contract = Box::new(ContractWrapper::new(
        mock_modules::standalone_cw2::mock_execute,
        mock_modules::standalone_cw2::mock_instantiate,
        mock_modules::standalone_cw2::mock_query,
    ));
    let standalone1_id = chain.app.borrow_mut().store_code(standalone1_contract);

    let standalone2_contract = Box::new(ContractWrapper::new(
        mock_modules::standalone_no_cw2::mock_execute,
        mock_modules::standalone_no_cw2::mock_instantiate,
        mock_modules::standalone_no_cw2::mock_query,
    ));
    let standalone2_id = chain.app.borrow_mut().store_code(standalone2_contract);

    // install first standalone
    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("abstract")?,
            name: "standalone1".to_owned(),
            version: ModuleVersion::Version(mock_modules::V1.to_owned()),
        },
        ModuleReference::Standalone(standalone1_id),
    )])?;

    account.install_module("abstract:standalone1", Some(&MockInitMsg {}), None)?;

    // install second standalone
    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("abstract")?,
            name: "standalone2".to_owned(),
            version: ModuleVersion::Version(mock_modules::V1.to_owned()),
        },
        ModuleReference::Standalone(standalone2_id),
    )])?;

    account.install_module("abstract:standalone2", Some(&MockInitMsg {}), None)?;
    take_storage_snapshot!(chain, "proxy_install_standalone_modules");
    Ok(())
}

#[test]
fn install_standalone_versions_not_met() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AbstractAccount::new(&deployment, AccountId::local(0));

    let standalone1_contract = Box::new(ContractWrapper::new(
        mock_modules::standalone_cw2::mock_execute,
        mock_modules::standalone_cw2::mock_instantiate,
        mock_modules::standalone_cw2::mock_query,
    ));
    let standalone1_id = chain.app.borrow_mut().store_code(standalone1_contract);

    // install first standalone
    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("abstract")?,
            name: "standalone1".to_owned(),
            version: ModuleVersion::Version(mock_modules::V2.to_owned()),
        },
        ModuleReference::Standalone(standalone1_id),
    )])?;

    let err = account
        .install_module("abstract:standalone1", Some(&MockInitMsg {}), None)
        .unwrap_err();

    if let AbstractInterfaceError::Orch(err) = err {
        let err: ManagerError = err.downcast()?;
        assert_eq!(
            err,
            ManagerError::Abstract(abstract_std::AbstractError::UnequalModuleData {
                cw2: mock_modules::V1.to_owned(),
                module: mock_modules::V2.to_owned(),
            })
        );
    } else {
        panic!("wrong error type")
    };

    Ok(())
}

#[test]
fn install_multiple_modules() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    chain.add_balance(&sender, vec![coin(86, "token1"), coin(500, "token2")])?;
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AbstractAccount::new(&deployment, ABSTRACT_ACCOUNT_ID);

    let standalone1_contract = Box::new(ContractWrapper::new(
        mock_modules::standalone_cw2::mock_execute,
        mock_modules::standalone_cw2::mock_instantiate,
        mock_modules::standalone_cw2::mock_query,
    ));
    let standalone1_id = chain.app.borrow_mut().store_code(standalone1_contract);

    let standalone2_contract = Box::new(ContractWrapper::new(
        mock_modules::standalone_no_cw2::mock_execute,
        mock_modules::standalone_no_cw2::mock_instantiate,
        mock_modules::standalone_no_cw2::mock_query,
    ));
    let standalone2_id = chain.app.borrow_mut().store_code(standalone2_contract);

    // install both standalone
    deployment.version_control.propose_modules(vec![
        (
            ModuleInfo {
                namespace: Namespace::new("abstract")?,
                name: "standalone1".to_owned(),
                version: ModuleVersion::Version(mock_modules::V1.to_owned()),
            },
            ModuleReference::Standalone(standalone1_id),
        ),
        (
            ModuleInfo {
                namespace: Namespace::new("abstract")?,
                name: "standalone2".to_owned(),
                version: ModuleVersion::Version(mock_modules::V1.to_owned()),
            },
            ModuleReference::Standalone(standalone2_id),
        ),
    ])?;

    // add monetization on module1
    let monetization = Monetization::InstallFee(FixedFee::new(&coin(42, "token1")));
    deployment.version_control.update_module_configuration(
        "standalone1".to_owned(),
        Namespace::new("abstract").unwrap(),
        UpdateModule::Versioned {
            version: mock_modules::V1.to_owned(),
            metadata: None,
            monetization: Some(monetization),
            instantiation_funds: None,
        },
    )?;

    // add init funds on module2
    deployment.version_control.update_module_configuration(
        "standalone2".to_owned(),
        Namespace::new("abstract").unwrap(),
        UpdateModule::Versioned {
            version: mock_modules::V1.to_owned(),
            metadata: None,
            monetization: None,
            instantiation_funds: Some(vec![coin(42, "token1"), coin(500, "token2")]),
        },
    )?;

    // Don't allow to attach too much funds
    let err = account
        .install_modules(
            vec![
                ModuleInstallConfig::new(
                    ModuleInfo::from_id_latest("abstract:standalone1")?,
                    Some(to_json_binary(&MockInitMsg {}).unwrap()),
                ),
                ModuleInstallConfig::new(
                    ModuleInfo::from_id_latest("abstract:standalone2")?,
                    Some(to_json_binary(&MockInitMsg {}).unwrap()),
                ),
            ],
            Some(&[coin(86, "token1"), coin(500, "token2")]),
        )
        .unwrap_err();
    assert!(err.root().to_string().contains(&format!(
        "Expected {:?}, sent {:?}",
        vec![coin(84, "token1"), coin(500, "token2")],
        vec![coin(86, "token1"), coin(500, "token2")]
    )));

    // successful install
    account.install_modules_auto(vec![
        ModuleInstallConfig::new(
            ModuleInfo::from_id_latest("abstract:standalone1")?,
            Some(to_json_binary(&MockInitMsg {}).unwrap()),
        ),
        ModuleInstallConfig::new(
            ModuleInfo::from_id_latest("abstract:standalone2")?,
            Some(to_json_binary(&MockInitMsg {}).unwrap()),
        ),
    ])?;

    // Make sure all installed
    let account_module_versions = account.manager.module_versions(vec![
        String::from("abstract:standalone1"),
        String::from("abstract:standalone2"),
    ])?;
    assert_eq!(
        account_module_versions,
        ModuleVersionsResponse {
            versions: vec![
                cw2::ContractVersion {
                    contract: String::from("abstract:standalone1"),
                    version: String::from(mock_modules::V1),
                },
                // Second doesn't have cw2
                cw2::ContractVersion {
                    contract: String::from("abstract:standalone2"),
                    version: String::from(mock_modules::V1),
                },
            ]
        }
    );

    let account_module_addresses = account.manager.module_addresses(vec![
        String::from("abstract:standalone1"),
        String::from("abstract:standalone2"),
    ])?;
    let (standalone_addr1, standalone_addr2) = match &account_module_addresses.modules[..] {
        [(_app1, addr1), (_app2, addr2)] => (addr1.clone(), addr2.clone()),
        _ => panic!("bad result from module_addresses"),
    };
    let s1_balance = chain.query_all_balances(&standalone_addr1)?;
    let s2_balance = chain.query_all_balances(&standalone_addr2)?;

    assert!(s1_balance.is_empty());
    assert_eq!(s2_balance, vec![coin(42, "token1"), coin(500, "token2")]);
    take_storage_snapshot!(chain, "proxy_install_multiple_modules");

    Ok(())
}

#[test]
fn renounce_cleans_namespace() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let account = deployment.account_factory.create_new_account(
        AccountDetails {
            name: "foo".to_string(),
            description: None,
            link: None,
            namespace: Some("bar".to_owned()),
            base_asset: None,
            install_modules: vec![],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        None,
    )?;

    let namespace_result = deployment
        .version_control
        .namespace(Namespace::unchecked("bar"));
    assert!(namespace_result.is_ok());

    account
        .manager
        .update_ownership(cw_ownable::Action::RenounceOwnership)?;

    let namespace_result = deployment
        .version_control
        .namespace(Namespace::unchecked("bar"))?;
    assert_eq!(namespace_result, NamespaceResponse::Unclaimed {});

    // Governance is in fact renounced
    let acc_cfg: InfoResponse = account.manager.info()?;
    assert_eq!(
        acc_cfg.info.governance_details,
        GovernanceDetails::Renounced {}
    );

    let account_owner = account.manager.ownership()?;
    assert!(account_owner.owner.is_none());
    Ok(())
}

#[test]
fn can_take_any_last_two_billion_accounts() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    deployment.account_factory.create_new_account(
        AccountDetails {
            name: "foo".to_string(),
            description: None,
            link: None,
            namespace: Some("bar".to_owned()),
            base_asset: None,
            install_modules: vec![],
            account_id: Some(2147483648),
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        None,
    )?;

    let already_exists = deployment.account_factory.create_new_account(
        AccountDetails {
            name: "foo".to_string(),
            description: None,
            link: None,
            namespace: Some("bar".to_owned()),
            base_asset: None,
            install_modules: vec![],
            // same id
            account_id: Some(2147483648),
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        None,
    );

    assert!(already_exists.is_err());
    Ok(())
}
