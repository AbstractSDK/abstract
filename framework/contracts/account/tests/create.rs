use abstract_interface::*;
use abstract_std::{
    account,
    objects::{
        account::AccountTrace, gov_type::GovernanceDetails, namespace::Namespace, AccountId,
    },
    registry::{self, Account, NamespaceInfo, NamespaceResponse},
    ABSTRACT_EVENT_TYPE, ACCOUNT,
};
use abstract_testing::prelude::*;
use cw_orch::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

#[test]
fn instantiate() -> AResult {
    let chain = MockBech32::new("mock");
    let deployment = Abstract::deploy_on(chain.clone(), ())?;

    let vc = deployment.registry;
    let vc_config = vc.config()?;
    let expected = abstract_std::registry::ConfigResponse {
        // Admin Account is ID 0
        local_account_sequence: 1,
        security_enabled: false,
        namespace_registration_fee: None,
    };

    assert_eq!(vc_config, expected);
    Ok(())
}

#[test]
fn create_one_account() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;

    let registry = &deployment.registry;

    let account = AccountI::new(ACCOUNT, chain);

    let account_creation = account.instantiate(
        &account::InstantiateMsg {
            code_id: 1,
            name: Some(String::from("first_account")),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            authenticator: None,
        },
        None,
        &[],
    )?;

    let account = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;

    let registry_config = registry.config()?;
    let expected = registry::ConfigResponse {
        local_account_sequence: 2,
        security_enabled: false,
        namespace_registration_fee: None,
    };
    assert_eq!(registry_config, expected);

    let account_list = registry.account(TEST_ACCOUNT_ID)?;

    assert_eq!(account_list, Account::new(Addr::unchecked(account)));

    Ok(())
}

#[test]
fn create_two_accounts() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;

    let registry = &deployment.registry;

    let account = AccountI::new(ACCOUNT, chain);
    // first account
    let account_1 = account.instantiate(
        &account::InstantiateMsg {
            code_id: 1,
            name: Some(String::from("first_account")),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            authenticator: None,
        },
        None,
        &[],
    )?;

    // second account
    let account_2 = account.instantiate(
        &account::InstantiateMsg {
            code_id: 1,
            name: Some(String::from("second_account")),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            authenticator: None,
        },
        None,
        &[],
    )?;

    let account1 = account_1.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    let account_1_id = TEST_ACCOUNT_ID;

    let account2 = account_2.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    let account_2_id = AccountId::new(TEST_ACCOUNT_ID.seq() + 1, AccountTrace::Local)?;

    let registry_config = registry.config()?;
    let expected = registry::ConfigResponse {
        namespace_registration_fee: None,
        security_enabled: false,
        // we created two accounts
        local_account_sequence: account_2_id.seq() + 1,
    };

    assert_eq!(&registry_config, &expected);

    let account_1 = registry.account(account_1_id)?;
    assert_eq!(account_1, Account::new(Addr::unchecked(account1)));

    let account_2 = registry.account(account_2_id)?;
    assert_eq!(account_2, Account::new(Addr::unchecked(account2)));

    Ok(())
}

#[test]
fn sender_is_not_admin_monarchy() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let account = AccountI::new(ACCOUNT, chain);

    let registry = &deployment.registry;
    let account_creation = account.instantiate(
        &account::InstantiateMsg {
            code_id: 1,
            name: Some(String::from("first_account")),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            authenticator: None,
        },
        None,
        &[],
    )?;

    let account_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    account.set_address(&Addr::unchecked(&account_addr));

    let registered_account = registry.account(TEST_ACCOUNT_ID)?;

    assert_eq!(account_addr, registered_account.addr().to_string());

    let account_config = account.config()?;

    assert_eq!(
        account_config,
        abstract_std::account::ConfigResponse {
            account_id: TEST_ACCOUNT_ID,
            registry_address: registry.address()?,
            module_factory_address: deployment.module_factory.address()?,
            is_suspended: false,
            whitelisted_addresses: vec![],
        }
    );

    Ok(())
}

#[test]
fn sender_is_not_admin_external() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let account = AccountI::new(ACCOUNT, chain);
    let registry = &deployment.registry;

    let account_creation = account.instantiate(
        &account::InstantiateMsg {
            code_id: 1,
            name: Some(String::from("first_account")),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::External {
                governance_address: sender.to_string(),
                governance_type: "some-gov-type".to_string(),
            },
            authenticator: None,
        },
        None,
        &[],
    )?;

    let account_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    account.set_address(&Addr::unchecked(&account_addr));

    let account_config = account.config()?;

    assert_eq!(
        account_config,
        abstract_std::account::ConfigResponse {
            account_id: TEST_ACCOUNT_ID,
            is_suspended: false,
            registry_address: registry.address()?,
            module_factory_address: deployment.module_factory.address()?,
            whitelisted_addresses: vec![],
        }
    );

    Ok(())
}

#[test]
fn create_one_account_with_namespace() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), ())?;
    let account = AccountI::new(ACCOUNT, chain);

    let namespace_to_claim = "namespace-to-claim";
    let account_creation = account.instantiate(
        &account::InstantiateMsg {
            code_id: 1,
            name: Some(String::from("first_account")),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(namespace_to_claim.to_string()),
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::External {
                governance_address: sender.to_string(),
                governance_type: "some-gov-type".to_string(),
            },
            authenticator: None,
        },
        None,
        &[],
    )?;

    let account_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    account.set_address(&Addr::unchecked(&account_addr));

    let account_config = account.config()?;

    assert_eq!(
        account_config,
        abstract_std::account::ConfigResponse {
            account_id: TEST_ACCOUNT_ID,
            is_suspended: false,
            registry_address: deployment.registry.address()?,
            module_factory_address: deployment.module_factory.address()?,
            whitelisted_addresses: vec![],
        }
    );
    // We need to check if the namespace is associated with this account
    let namespace = deployment
        .registry
        .namespace(Namespace::new(namespace_to_claim)?)?;

    assert_eq!(
        namespace,
        NamespaceResponse::Claimed(NamespaceInfo {
            account_id: TEST_ACCOUNT_ID,
            account: Account::new(Addr::unchecked(account_addr)),
        })
    );

    Ok(())
}

#[test]
fn create_one_account_with_namespace_fee() -> AResult {
    let chain = MockBech32::new("mock");
    Abstract::deploy_on(chain.clone(), ())?;
    abstract_integration_tests::create::create_one_account_with_namespace_fee(chain)
}
