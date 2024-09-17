use abstract_interface::*;
use abstract_std::{
    account,
    objects::{
        account::AccountTrace, gov_type::GovernanceDetails, namespace::Namespace, AccountId,
    },
    version_control::{self, Account, NamespaceInfo, NamespaceResponse},
    ABSTRACT_EVENT_TYPE, ACCOUNT,
};
use abstract_testing::prelude::*;
use cw_orch::prelude::*;
use speculoos::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

#[test]
fn instantiate() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let vc = deployment.version_control;
    let vc_config = vc.config()?;
    let expected = abstract_std::version_control::ConfigResponse {
        // Admin Account is ID 0
        local_account_sequence: 1,
        security_disabled: true,
        namespace_registration_fee: None,
    };

    assert_that!(&vc_config).is_equal_to(&expected);
    Ok(())
}

#[test]
fn create_one_account() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let version_control = &deployment.version_control;

    let account = AccountI::new(ACCOUNT, chain);

    let account_creation = account.instantiate(
        &account::InstantiateMsg {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            module_factory_address: deployment.module_factory.addr_str()?,
            version_control_address: version_control.addr_str()?,
        },
        None,
        &[],
    )?;

    let account = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;

    let version_control_config = version_control.config()?;
    let expected = version_control::ConfigResponse {
        local_account_sequence: 2,
        security_disabled: true,
        namespace_registration_fee: None,
    };

    assert_that!(&version_control_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_std::version_control::ConfigResponse {
        local_account_sequence: 2,
        security_disabled: true,
        namespace_registration_fee: Default::default(),
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let account_list = version_control.account(TEST_ACCOUNT_ID)?;

    assert_that!(&account_list.account_base.into()).is_equal_to(Account::new(account));

    Ok(())
}

#[test]
fn create_two_account_s() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let version_control = &deployment.version_control;

    let account = AccountI::new(ACCOUNT, chain);
    // first account
    let account_1 = account.instantiate(
        &account::InstantiateMsg {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            module_factory_address: deployment.module_factory.addr_str()?,
            version_control_address: version_control.addr_str()?,
        },
        None,
        &[],
    )?;

    // second account
    let account_2 = account.instantiate(
        &account::InstantiateMsg {
            name: String::from("second_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            module_factory_address: deployment.module_factory.addr_str()?,
            version_control_address: version_control.addr_str()?,
        },
        None,
        &[],
    )?;

    let account1 = account_1.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    let account_1_id = TEST_ACCOUNT_ID;

    let account2 = account_2.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    let account_2_id = AccountId::new(TEST_ACCOUNT_ID.seq() + 1, AccountTrace::Local)?;

    let version_control_config = version_control.config()?;
    let expected = version_control::ConfigResponse {
        namespace_registration_fee: None,
        security_disabled: true,
        // we created two accounts
        local_account_sequence: account_2_id.seq() + 1,
    };

    assert_that!(&version_control_config).is_equal_to(&expected);

    let account_1 = version_control.account(account_1_id)?.account_base;
    assert_that!(account_1.into()).is_equal_to(Account::new(account1));

    let account_2 = version_control.account(account_2_id)?.account_base;
    assert_that!(account_2.into()).is_equal_to(Account::new(account2));

    Ok(())
}

#[test]
fn sender_is_not_admin_monarchy() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AccountI::new(ACCOUNT, chain);

    let version_control = &deployment.version_control;
    let account_creation = account.instantiate(
        &account::InstantiateMsg {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::Monarchy {
                monarch: sender.to_string(),
            },
            module_factory_address: deployment.module_factory.addr_str()?,
            version_control_address: version_control.addr_str()?,
        },
        None,
        &[],
    )?;

    let account_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    account.set_address(&Addr::unchecked(&account_addr));

    let registered_account = version_control.account(TEST_ACCOUNT_ID)?.account_base;

    assert_that!(account_addr).is_equal_to(registered_account.addr().to_string());

    let account_config = account.config()?;

    assert_that!(account_config).is_equal_to(abstract_std::account::ConfigResponse {
        account_id: TEST_ACCOUNT_ID,
        version_control_address: version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        is_suspended: false,
        modules: vec![],
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_external() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AccountI::new(ACCOUNT, chain);
    let version_control = &deployment.version_control;

    let account_creation = account.instantiate(
        &account::InstantiateMsg {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::External {
                governance_address: sender.to_string(),
                governance_type: "some-gov-type".to_string(),
            },
            module_factory_address: deployment.module_factory.addr_str()?,
            version_control_address: version_control.addr_str()?,
        },
        None,
        &[],
    )?;

    let account_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    account.set_address(&Addr::unchecked(&account_addr));

    let account_config = account.config()?;

    assert_that!(account_config).is_equal_to(abstract_std::account::ConfigResponse {
        account_id: TEST_ACCOUNT_ID,
        is_suspended: false,
        version_control_address: version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        modules: vec![],
    });

    Ok(())
}

#[test]
fn create_one_account_with_namespace() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;
    let account = AccountI::new(ACCOUNT, chain);

    let namespace_to_claim = "namespace-to-claim";
    let account_creation = account.instantiate(
        &account::InstantiateMsg {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(namespace_to_claim.to_string()),
            install_modules: vec![],
            account_id: None,
            owner: GovernanceDetails::External {
                governance_address: sender.to_string(),
                governance_type: "some-gov-type".to_string(),
            },
            module_factory_address: deployment.module_factory.addr_str()?,
            version_control_address: deployment.version_control.addr_str()?,
        },
        None,
        &[],
    )?;

    let account_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    account.set_address(&Addr::unchecked(&account_addr));

    let account_config = account.config()?;

    assert_that!(account_config).is_equal_to(abstract_std::account::ConfigResponse {
        account_id: TEST_ACCOUNT_ID,
        is_suspended: false,
        version_control_address: deployment.version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        modules: vec![],
    });
    // We need to check if the namespace is associated with this account
    let namespace = deployment
        .version_control
        .namespace(Namespace::new(namespace_to_claim)?)?;

    assert_that!(&namespace).is_equal_to(&NamespaceResponse::Claimed(NamespaceInfo {
        account_id: TEST_ACCOUNT_ID,
        account_base: Account::new(Addr::unchecked(account_addr)),
    }));

    Ok(())
}

#[test]
fn create_one_account_with_namespace_fee() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    Abstract::deploy_on(chain.clone(), sender.to_string())?;
    abstract_integration_tests::create::create_one_account_with_namespace_fee(chain)
}
