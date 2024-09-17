use abstract_interface::{AbstractAccount, VCQueryFns, *};
use abstract_std::{
    objects::{
        account::AccountTrace, gov_type::GovernanceDetails, namespace::Namespace, AccountId,
    },
    version_control::{self, Account, NamespaceInfo, NamespaceResponse},
    ABSTRACT_EVENT_TYPE,
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
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let version_control = &deployment.version_control;

    let account_creation = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            install_modules: vec![],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        // Account creation fee not covered
        &[],
    );

    let account = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;

    let version_control_config = version_control.config()?;
    let expected = version_control::ConfigResponse {
        security_disabled: todo!(),
        namespace_registration_fee: todo!(),
    };

    assert_that!(&version_control_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_std::version_control::ConfigResponse {
        security_disabled: true,
        namespace_registration_fee: Default::default(),
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let account_list = version_control.account_base(TEST_ACCOUNT_ID)?;

    assert_that!(&account_list.account_base).is_equal_to(Account {
        manager: Addr::unchecked(manager),
        proxy: Addr::unchecked(proxy),
    });

    Ok(())
}

#[test]
fn create_two_account_s() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let version_control = &deployment.account_factory;
    let version_control = &deployment.version_control;
    // first account
    let account_1 = version_control.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_os"),
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;
    // second account
    let account_2 = version_control.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("second_os"),
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;

    let account1 = account_1.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    let account_1_id = TEST_ACCOUNT_ID;

    let account2 = account_2.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;
    let account_2_id = AccountId::new(TEST_ACCOUNT_ID.seq() + 1, AccountTrace::Local)?;

    let version_control_config = version_control.config()?;
    let expected = version_control::ConfigResponse {
        ans_host_contract: deployment.ans_host.address()?,
        version_control_contract: deployment.version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        // we created two accounts
        local_account_sequence: account_2_id.seq() + 1,
    };

    assert_that!(&version_control_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_std::version_control::ConfigResponse {
        account_factory_address: Some(version_control.address()?),
        security_disabled: true,
        namespace_registration_fee: Default::default(),
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let account_1 = version_control.account_base(account_1_id)?.account_base;
    assert_that!(&account_1).is_equal_to(Account {
        manager: Addr::unchecked(account1),
        proxy: Addr::unchecked(proxy1),
    });

    let account_2 = version_control.account_base(account_2_id)?.account_base;
    assert_that!(&account_2).is_equal_to(Account {
        manager: Addr::unchecked(manager2),
        proxy: Addr::unchecked(proxy2),
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_monarchy() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let version_control = &deployment.account_factory;
    let version_control = &deployment.version_control;
    let account_creation = version_control.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_os"),
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;

    let manager = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;

    let account = version_control.account_base(TEST_ACCOUNT_ID)?.account_base;

    let account_1 = AbstractAccount::new(&deployment, TEST_ACCOUNT_ID);
    assert_that!(Account {
        manager: account_1.account.address()?,
        proxy: account_1.proxy.address()?,
    })
    .is_equal_to(&account);

    assert_that!(Account {
        manager: Addr::unchecked(manager),
        proxy: Addr::unchecked(proxy),
    })
    .is_equal_to(&account);

    let account_config = account_1.account.config()?;

    assert_that!(account_config).is_equal_to(abstract_std::manager::ConfigResponse {
        account_id: TEST_ACCOUNT_ID,
        version_control_address: version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        is_suspended: false,
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_external() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let version_control = &deployment.account_factory;
    let version_control = &deployment.version_control;
    version_control.create_account(
        GovernanceDetails::External {
            governance_address: sender.to_string(),
            governance_type: "some-gov-type".to_string(),
        },
        vec![],
        String::from("first_os"),
        None,
        Some(String::from("account_description")),
        Some(String::from("http://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;

    let account = AbstractAccount::new(&deployment, TEST_ACCOUNT_ID);
    let account_config = account.account.config()?;

    assert_that!(account_config).is_equal_to(abstract_std::manager::ConfigResponse {
        account_id: TEST_ACCOUNT_ID,
        is_suspended: false,
        version_control_address: version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
    });

    Ok(())
}

#[test]
fn create_one_account_with_namespace() -> AResult {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let version_control = &deployment.account_factory;
    let version_control = &deployment.version_control;

    let namespace_to_claim = "namespace-to-claim";

    let account_creation = version_control.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_account"),
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        Some(namespace_to_claim.to_string()),
        &[],
    )?;

    let manager_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "account_address")?;

    // We need to check if the namespace is associated with this account
    let namespace = version_control.namespace(Namespace::new(namespace_to_claim)?)?;

    assert_that!(&namespace).is_equal_to(&NamespaceResponse::Claimed(NamespaceInfo {
        account_id: TEST_ACCOUNT_ID,
        account_base: Account {
            manager: Addr::unchecked(manager_addr),
            proxy: Addr::unchecked(proxy_addr),
        },
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
