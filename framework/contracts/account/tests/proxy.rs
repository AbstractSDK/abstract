use abstract_account::{contract::CONTRACT_VERSION, error::AccountError};
use abstract_adapter::mock::{MockExecMsg, MockInitMsg};
use abstract_integration_tests::*;
use abstract_interface::*;
use abstract_std::{
    account::{AccountModuleInfo, ModuleInstallConfig, ModuleVersionsResponse},
    objects::{
        fee::FixedFee,
        gov_type::{GovAction, GovernanceDetails},
        module::{ModuleInfo, ModuleVersion, Monetization},
        module_reference::ModuleReference,
        namespace::Namespace,
        ownership::{self, cw721::OwnerOfResponse},
        AccountId, ABSTRACT_ACCOUNT_ID,
    },
    version_control::{NamespaceResponse, UpdateModule},
    ACCOUNT,
};
use abstract_testing::prelude::*;
use anyhow::Error;
use cosmwasm_std::{coin, CosmosMsg, Uint128};
use cw_orch::{
    mock::{cw_multi_test::MockApiBech32, MockBase},
    prelude::*,
};
use speculoos::prelude::*;

// /// Deploys and mints an NFT to *sender*.
// fn deploy_and_mint_nft(
//     chain: MockBase<MockApiBech32>,
//     sender: Addr,
// ) -> Result<(String, Addr), Error> {
//     let token_id = String::from("1");

//     let cw721_contract = Box::new(ContractWrapper::new(
//         cw721_base::entry::execute,
//         cw721_base::entry::instantiate,
//         cw721_base::entry::query,
//     ));
//     let cw721_id = chain.app.borrow_mut().store_code(cw721_contract);

//     // instantiate mock collection
//     let res = chain.instantiate(
//         cw721_id,
//         &cw721_base::InstantiateMsg {
//             name: "testcollection".to_string(),
//             symbol: "TEST".to_string(),
//             minter: sender.to_string(),
//         },
//         Some("test-account-nft-collection"),
//         Some(&sender),
//         &[],
//     )?;

//     let nft_addr = res.instantiated_contract_address()?;

//     mint_nft(&chain, sender, token_id.clone(), &nft_addr)?;
//     Ok((token_id, nft_addr))
// }

// fn mint_nft(
//     chain: &MockBase<MockApiBech32>,
//     owner: impl Into<String>,
//     token_id: impl Into<String>,
//     nft_addr: &Addr,
// ) -> anyhow::Result<()> {
//     chain.execute(
//         &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint {
//             token_id: token_id.into(),
//             owner: owner.into(),
//             token_uri: None,
//             extension: None,
//         },
//         &[],
//         nft_addr,
//     )?;
//     Ok(())
// }

#[test]
fn instantiate() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&sender, &deployment)?;

    let modules = account.module_infos(None, None)?.module_infos;

    // assert proxy module
    assert_that!(&modules).has_length(0);

    // assert account config
    assert_that!(account.config()?).is_equal_to(abstract_std::account::ConfigResponse {
        whitelisted_addresses: vec![],
        version_control_address: deployment.version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        account_id: TEST_ACCOUNT_ID,
        is_suspended: false,
    });
    take_storage_snapshot!(chain, "instantiate_account");
    Ok(())
}

#[test]
fn exec_on_account() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    // This testing environments allows you to use simple deploy contraptions:
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&sender, &deployment)?;

    // Mint coins to proxy address
    chain.set_balance(&account.address()?, vec![Coin::new(100_000u128, TTOKEN)])?;

    let proxy_balance = chain.bank_querier().balance(&account.address()?, None)?;

    assert_that!(proxy_balance).is_equal_to(vec![Coin::new(100_000u128, TTOKEN)]);

    let burn_amount = vec![Coin::new(10_000u128, TTOKEN)];

    // Burn coins from proxy
    account.module_action(vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
        amount: burn_amount,
    })])?;

    // Assert balance has decreased
    let proxy_balance = chain.bank_querier().balance(&account.address()?, None)?;
    assert_that!(proxy_balance).is_equal_to(vec![Coin::new((100_000 - 10_000) as u128, TTOKEN)]);
    take_storage_snapshot!(chain, "exec_on_account");

    Ok(())
}

#[test]
fn default_without_response_data() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = create_default_account(&sender, &deployment)?;
    let _staking_adapter_one = init_mock_adapter(chain.clone(), &deployment, None, account.id()?)?;

    install_adapter(&account, TEST_MODULE_ID)?;

    chain.set_balance(&account.address()?, vec![Coin::new(100_000u128, TTOKEN)])?;

    let resp = account.execute_on_module(
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
    let sender = chain.sender_addr();
    Abstract::deploy_on(chain.clone(), sender.to_string())?;
    abstract_integration_tests::account::with_response_data(chain.clone())?;
    take_storage_snapshot!(chain, "proxy_with_response_data");

    Ok(())
}

#[test]
fn install_standalone_modules() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AccountI::load_from(&deployment, AccountId::local(0))?;

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

    account.install_module("abstract:standalone1", Some(&MockInitMsg {}), &[])?;

    // install second standalone
    deployment.version_control.propose_modules(vec![(
        ModuleInfo {
            namespace: Namespace::new("abstract")?,
            name: "standalone2".to_owned(),
            version: ModuleVersion::Version(mock_modules::V1.to_owned()),
        },
        ModuleReference::Standalone(standalone2_id),
    )])?;

    account.install_module("abstract:standalone2", Some(&MockInitMsg {}), &[])?;
    take_storage_snapshot!(chain, "proxy_install_standalone_modules");
    Ok(())
}

#[test]
fn install_standalone_versions_not_met() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AccountI::load_from(&deployment, AccountId::local(0))?;

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
        .install_module("abstract:standalone1", Some(&MockInitMsg {}), &[])
        .unwrap_err();

    if let AbstractInterfaceError::Orch(err) = err {
        let err: AccountError = err.downcast()?;
        assert_eq!(
            err,
            AccountError::Abstract(abstract_std::AbstractError::UnequalModuleData {
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
    let sender = chain.sender_addr();
    chain.add_balance(&sender, vec![coin(86, "token1"), coin(500, "token2")])?;
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AccountI::load_from(&deployment, ABSTRACT_ACCOUNT_ID)?;

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
            &[coin(86, "token1"), coin(500, "token2")],
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
    let account_module_versions = account.module_versions(vec![
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

    let account_module_addresses = account.module_addresses(vec![
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
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let account = AccountI::create(
        &deployment,
        AccountDetails {
            name: "foo".to_string(),
            description: None,
            link: None,
            namespace: Some("bar".to_owned()),
            install_modules: vec![],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    )?;

    let namespace_result = deployment
        .version_control
        .namespace(Namespace::unchecked("bar"));
    assert!(namespace_result.is_ok());

    account.update_ownership(ownership::GovAction::RenounceOwnership)?;

    let namespace_result = deployment
        .version_control
        .namespace(Namespace::unchecked("bar"))?;
    assert_eq!(namespace_result, NamespaceResponse::Unclaimed {});

    // Governance is in fact renounced
    let ownership = account.ownership()?;
    assert_eq!(ownership.owner, GovernanceDetails::Renounced {});

    Ok(())
}

// #[test]
// fn nft_owner_success() -> Result<(), Error> {
//     let chain = MockBech32::new("mock");
//     let sender = chain.sender_addr();
//     let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
//     let (token_id, nft_addr) = deploy_and_mint_nft(chain.clone(), sender.clone())?;

//     let gov = GovernanceDetails::NFT {
//         collection_addr: nft_addr.to_string(),
//         // token minted to sender
//         token_id: token_id.clone(),
//     };

//     // create nft-owned account
//     let account = AccountI::create(
//         &deployment,
//         AccountDetails {
//             name: "foo".to_string(),
//             description: None,
//             link: None,
//             namespace: None,
//             install_modules: vec![],
//             account_id: None,
//         },
//         gov,
//         &[],
//     )?;

//     let start_amnt = 100_000u128;
//     let burn_amnt = 10_000u128;
//     let start_balance = vec![Coin::new(start_amnt, TTOKEN)];
//     let burn_amount: Vec<Coin> = vec![Coin::new(burn_amnt, TTOKEN)];

//     let first_burn = Uint128::from(start_amnt).checked_sub(burn_amnt.into())?;

//     // fund nft account
//     chain.set_balance(&account.address()?, start_balance.clone())?;

//     // test sending msg as nft account by burning tokens from proxy
//     let burn_msg = abstract_std::account::ExecuteMsg::ModuleAction {
//         msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Burn {
//             amount: burn_amount,
//         })],
//     };

//     // confirm sender (who owns this NFT) can execute on the account through the account
//     account.execute_on_module(ACCOUNT, &burn_msg)?;

//     // confirm tokens were burnt
//     let balance = chain.query_balance(&account.address()?, TTOKEN)?;

//     assert_eq!(balance.clone(), first_burn.clone());

//     // confirm only token holder can send msg
//     let not_nft_holder = chain.addr_make_with_balance("test", vec![])?;

//     let res = account
//         .call_as(&not_nft_holder)
//         .execute_on_module(ACCOUNT, &burn_msg);

//     assert!(&res.is_err());

//     // Now transfer the NFT
//     let new_nft_owner = not_nft_holder;

//     chain.execute(
//         &cw721::Cw721ExecuteMsg::TransferNft {
//             recipient: new_nft_owner.to_string(),
//             token_id: token_id.clone(),
//         },
//         &[],
//         &Addr::unchecked(nft_addr.clone()),
//     )?;

//     // ensure NFT was transferred
//     let resp: OwnerOfResponse = chain.wasm_querier().smart_query(
//         &nft_addr,
//         &cw721::Cw721QueryMsg::OwnerOf {
//             token_id: token_id.clone(),
//             include_expired: None,
//         },
//     )?;
//     assert_eq!(resp.owner, new_nft_owner.to_string());

//     // try to call as the old owner (default sender)
//     let res = account.execute_on_module(ACCOUNT, &burn_msg);
//     assert!(&res.is_err());

//     // Now try with new NFT owner
//     account
//         .call_as(&new_nft_owner)
//         .execute_on_module(ACCOUNT, burn_msg)?;

//     let balance = chain.query_balance(&account.address()?, TTOKEN)?;
//     assert_eq!(balance, first_burn.checked_sub(burn_amnt.into())?);

//     Ok(())
// }

// #[test]
// fn nft_owner_immutable() -> Result<(), Error> {
//     let chain = MockBech32::new("mock");
//     let sender = chain.sender_addr();
//     let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
//     let (token_id, nft_addr) = deploy_and_mint_nft(chain.clone(), sender.clone())?;

//     let gov = GovernanceDetails::NFT {
//         collection_addr: nft_addr.to_string(),
//         // token minted to sender
//         token_id: token_id.clone(),
//     };

//     // create nft-owned account
//     let account = AccountI::create(
//         &deployment,
//         AccountDetails {
//             name: "foo".to_string(),
//             description: None,
//             link: None,
//             namespace: None,
//             install_modules: vec![],
//             account_id: None,
//         },
//         gov,
//         &[],
//     )?;

//     let not_nft_owner = chain.addr_make("not_nft_owner");

//     // NFT owned account governance cannot be transferred
//     let err: AccountError = account
//         .update_ownership(GovAction::TransferOwnership {
//             new_owner: GovernanceDetails::Monarchy {
//                 monarch: not_nft_owner.to_string(),
//             },
//             expiry: None,
//         })
//         .unwrap_err()
//         .downcast()
//         .unwrap();
//     assert_eq!(
//         err,
//         AccountError::Ownership(ownership::GovOwnershipError::ChangeOfNftOwned)
//     );

//     // NFT owned account governance cannot be renounced
//     let err: AccountError = account
//         .update_ownership(ownership::GovAction::RenounceOwnership)
//         .unwrap_err()
//         .downcast()
//         .unwrap();
//     assert_eq!(
//         err,
//         AccountError::Ownership(ownership::GovOwnershipError::ChangeOfNftOwned)
//     );

//     // create nft-owned sub-account
//     let sub_account = account.create_and_return_sub_account(
//         AccountDetails {
//             name: "sub-foo".to_string(),
//             description: None,
//             link: None,
//             namespace: None,
//             install_modules: vec![],
//             account_id: None,
//         },
//         &[],
//     )?;

//     // NFT owned sub-account governance cannot be transferred
//     let err: AccountError = sub_account
//         .update_ownership(GovAction::TransferOwnership {
//             new_owner: GovernanceDetails::Monarchy {
//                 monarch: not_nft_owner.to_string(),
//             },
//             expiry: None,
//         })
//         .unwrap_err()
//         .downcast()
//         .unwrap();
//     assert_eq!(
//         err,
//         AccountError::Ownership(ownership::GovOwnershipError::ChangeOfNftOwned)
//     );

//     // NFT owned sub-account governance cannot be renounced
//     let err: AccountError = sub_account
//         .update_ownership(ownership::GovAction::RenounceOwnership)
//         .unwrap_err()
//         .downcast()
//         .unwrap();
//     assert_eq!(
//         err,
//         AccountError::Ownership(ownership::GovOwnershipError::ChangeOfNftOwned)
//     );

//     Ok(())
// }

// #[test]
// fn nft_pending_owner() -> Result<(), Error> {
//     let chain = MockBech32::new("mock");
//     let sender = chain.sender_addr();
//     let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
//     let (token_id, nft_addr) = deploy_and_mint_nft(chain.clone(), sender.clone())?;

//     let gov = GovernanceDetails::NFT {
//         collection_addr: nft_addr.to_string(),
//         // token minted to sender
//         token_id: token_id.clone(),
//     };

//     let account = AccountI::create_default_account(
//         &deployment,
//         GovernanceDetails::Monarchy {
//             monarch: chain.sender_addr().to_string(),
//         },
//     )?;
//     // Transferring to token id that pending governance don't own act same way as transferring to renounced governance
//     let err: AccountError = account
//         .update_ownership(GovAction::TransferOwnership {
//             new_owner: GovernanceDetails::NFT {
//                 collection_addr: nft_addr.to_string(),
//                 token_id: "falsy_token_id".to_owned(),
//             },
//             expiry: None,
//         })
//         .unwrap_err()
//         .downcast()
//         .unwrap();
//     assert_eq!(
//         err,
//         AccountError::Ownership(ownership::GovOwnershipError::TransferToRenounced)
//     );

//     // Now transfer to correct token id
//     account.update_ownership(GovAction::TransferOwnership {
//         new_owner: gov.clone(),
//         expiry: None,
//     })?;
//     // Burn nft, which will make it act like we don't have pending ownership
//     chain.execute(
//         &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Burn { token_id },
//         &[],
//         &nft_addr,
//     )?;
//     // Account have pending NFT governance
//     // Note that there is no pending period
//     let ownership = account.ownership()?;
//     assert_eq!(ownership.pending_owner.unwrap(), gov);

//     let err: AccountError = account
//         .update_ownership(GovAction::AcceptOwnership)
//         .unwrap_err()
//         .downcast()
//         .unwrap();
//     assert_eq!(
//         err,
//         AccountError::Ownership(ownership::GovOwnershipError::TransferNotFound)
//     );

//     // Mint new NFT, since we burned previous one
//     let new_token_id = "2".to_owned();
//     mint_nft(&chain, chain.sender_addr(), &new_token_id, &nft_addr)?;

//     // Propose NFT governance
//     account.update_ownership(GovAction::TransferOwnership {
//         new_owner: (GovernanceDetails::NFT {
//             collection_addr: nft_addr.to_string(),
//             // token minted to sender
//             token_id: new_token_id.clone(),
//         }),
//         expiry: None,
//     })?;

//     // Only NFT owner can accept it
//     let err: AccountError = account
//         .call_as(&chain.addr_make("not_nft_owner"))
//         .update_ownership(GovAction::AcceptOwnership)
//         .unwrap_err()
//         .downcast()
//         .unwrap();
//     assert_eq!(
//         err,
//         AccountError::Ownership(ownership::GovOwnershipError::NotPendingOwner)
//     );

//     // Now accept without accidents
//     account.update_ownership(GovAction::AcceptOwnership)?;

//     // Burn NFT, to ensure account becomes unusable
//     chain.execute(
//         &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Burn {
//             token_id: new_token_id,
//         },
//         &[],
//         &nft_addr,
//     )?;

//     let err: AccountError = account
//         .update_info(Some("RIP Account".to_owned()), None, None)
//         .unwrap_err()
//         .downcast()
//         .unwrap();
//     assert_eq!(
//         err,
//         AccountError::Ownership(ownership::GovOwnershipError::NoOwner)
//     );
//     Ok(())
// }

#[test]
fn can_take_any_last_two_billion_accounts() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    AccountI::create(
        &deployment,
        AccountDetails {
            name: "foo".to_string(),
            description: None,
            link: None,
            namespace: Some("bar".to_owned()),
            install_modules: vec![],
            account_id: Some(2147483648),
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    )?;

    let already_exists = AccountI::create(
        &deployment,
        AccountDetails {
            name: "foo".to_string(),
            description: None,
            link: None,
            namespace: Some("bar".to_owned()),
            install_modules: vec![],
            // same id
            account_id: Some(2147483648),
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    );

    assert!(already_exists.is_err());
    Ok(())
}

#[test]
fn increment_not_effected_by_claiming() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let next_account_id = deployment.version_control.config()?.local_account_sequence;
    assert_eq!(next_account_id, 1);

    AccountI::create(
        &deployment,
        AccountDetails {
            name: "foo".to_string(),
            description: None,
            link: None,
            namespace: Some("bar".to_owned()),
            install_modules: vec![],
            account_id: Some(2147483648),
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[],
    )?;

    let next_account_id = deployment.version_control.config()?.local_account_sequence;
    assert_eq!(next_account_id, 1);

    // create new account
    AccountI::create_default_account(
        &deployment,
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
    )?;

    let next_account_id = deployment.version_control.config()?.local_account_sequence;
    assert_eq!(next_account_id, 2);

    Ok(())
}
