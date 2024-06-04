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
use cosmwasm_std::{coin, testing::MOCK_CONTRACT_ADDR, wasm_execute, CosmosMsg};
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

#[test]
fn exec_through_manager() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&deployment.account_factory)?;

    // mint coins to proxy address
    chain.set_balance(&account.proxy.address()?, vec![Coin::new(100_000, TTOKEN)])?;

    // burn coins from proxy
    let proxy_balance = chain
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000, TTOKEN)]);

    let burn_amount: Vec<Coin> = vec![Coin::new(10_000, TTOKEN)];

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
        .app
        .borrow()
        .wrap()
        .query_all_balances(account.proxy.address()?)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000 - 10_000, TTOKEN)]);
    take_storage_snapshot!(chain, "exec_through_manager");

    Ok(())
}

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
fn test_nft_as_governance() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let mut test_nft_collection = String::default();
    let token_id = String::from("1");

    let cw721_contract = Box::new(ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    ));
    let cw721_id = chain.app.borrow_mut().store_code(cw721_contract);

    // instantiate mock collection
    let res = chain.instantiate(
        cw721_id,
        &cw721_base::InstantiateMsg {
            name: "testcollection".to_string(),
            symbol: "TEST".to_string(),
            minter: sender.to_string(),
        },
        Some("test-account-nft-collection"),
        Some(&sender),
        &vec![],
    )?;
    // get contract id
    for event in &res.events {
        for attribute in &event.attributes {
            if attribute.key.to_lowercase() == "_contract_address" {
                test_nft_collection = attribute.value.to_string();
            }
        }
    }
    println!("test_nft_collection: {:?}", test_nft_collection);
    // mint
    chain.execute(
        &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint {
            token_id: token_id.clone(),
            owner: sender.to_string(),
            token_uri: None,
            extension: None,
        },
        &[],
        &Addr::unchecked(test_nft_collection.clone()),
    )?;

    let account = AbstractAccount::new(&deployment, AccountId::local(0));

    let gov =  GovernanceDetails::NFT {
        collection_addr: test_nft_collection.clone(),
        token_id,
    };
    let name =  "test-nft-governance-account-1".to_string();

    deployment.account_factory.create_account(gov, vec![], name, None,None,None,None,None,&vec![])?;
    Ok(())
}
