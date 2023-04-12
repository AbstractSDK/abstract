mod common;
use abstract_boot::{
    AbstractAccount, AccountFactoryExecFns, AccountFactoryQueryFns, VCQueryFns, *,
};
use abstract_core::{
    account_factory, objects::gov_type::GovernanceDetails, version_control::AccountBase,
    ABSTRACT_EVENT_NAME,
};
use boot_core::{
    Deploy, IndexResponse, {instantiate_default_mock_env, ContractInstance},
};
use common::TEST_VERSION;
use cosmwasm_std::{Addr, Uint64};
use speculoos::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

#[test]
fn instantiate() -> AResult {
    let _not_owner = Addr::unchecked("not_owner");
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain, TEST_VERSION.parse().unwrap())?;

    let factory = deployment.account_factory;
    let factory_config = factory.config()?;
    let expected = account_factory::ConfigResponse {
        owner: sender,
        ans_host_contract: deployment.ans_host.address()?.into(),
        version_control_contract: deployment.version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
        next_account_id: 0,
    };

    assert_that!(&factory_config).is_equal_to(&expected);
    Ok(())
}

#[test]
fn create_one_os() -> AResult {
    let _not_owner = Addr::unchecked("not_owner");
    let sender = Addr::unchecked(common::OWNER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain, TEST_VERSION.parse().unwrap())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    let account_creation = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        String::from("first_os"),
        Some(String::from("account_description")),
        Some(String::from("account_link_of_at_least_11_char")),
    )?;

    let manager = account_creation.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
    let proxy = account_creation.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;

    let factory_config = factory.config()?;
    let expected = account_factory::ConfigResponse {
        owner: sender.clone(),
        ans_host_contract: deployment.ans_host.address()?.into(),
        version_control_contract: deployment.version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
        next_account_id: 1,
    };

    assert_that!(&factory_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_core::version_control::ConfigResponse {
        admin: sender,
        factory: factory.address()?,
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let account_list = version_control.account_base(0)?;

    assert_that!(&account_list.account_base).is_equal_to(AccountBase {
        manager: Addr::unchecked(manager),
        proxy: Addr::unchecked(proxy),
    });

    Ok(())
}

#[test]
fn create_two_account_s() -> AResult {
    let _not_owner = Addr::unchecked("not_owner");
    let sender = Addr::unchecked(common::OWNER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain, TEST_VERSION.parse().unwrap())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    // first account
    let account_1 = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        String::from("first_os"),
        Some(String::from("account_description")),
        Some(String::from("account_link_of_at_least_11_char")),
    )?;
    // second account
    let account_2 = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        String::from("second_os"),
        Some(String::from("account_description")),
        Some(String::from("account_link_of_at_least_11_char")),
    )?;

    let manager1 = account_1.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
    let proxy1 = account_1.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;

    let manager2 = account_2.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
    let proxy2 = account_2.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;

    let factory_config = factory.config()?;
    let expected = account_factory::ConfigResponse {
        owner: sender.clone(),
        ans_host_contract: deployment.ans_host.address()?.into(),
        version_control_contract: deployment.version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
        next_account_id: 2,
    };

    assert_that!(&factory_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_core::version_control::ConfigResponse {
        admin: sender,
        factory: factory.address()?,
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let account_1 = version_control.account_base(0)?.account_base;
    assert_that!(&account_1).is_equal_to(AccountBase {
        manager: Addr::unchecked(manager1),
        proxy: Addr::unchecked(proxy1),
    });

    let account_2 = version_control.account_base(1)?.account_base;
    assert_that!(&account_2).is_equal_to(AccountBase {
        manager: Addr::unchecked(manager2),
        proxy: Addr::unchecked(proxy2),
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_monarchy() -> AResult {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked(common::OWNER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    let account_creation = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: owner.to_string(),
        },
        String::from("first_os"),
        Some(String::from("account_description")),
        Some(String::from("account_link_of_at_least_11_char")),
    )?;

    let manager = account_creation.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
    let proxy = account_creation.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;

    let account = version_control.account_base(0)?.account_base;

    let account_1 = AbstractAccount::new(chain, Some(0));
    assert_that!(AccountBase {
        manager: account_1.manager.address()?,
        proxy: account_1.proxy.address()?,
    })
    .is_equal_to(&account);

    assert_that!(AccountBase {
        manager: Addr::unchecked(manager),
        proxy: Addr::unchecked(proxy),
    })
    .is_equal_to(&account);

    let account_config = account_1.manager.config()?;

    assert_that!(account_config).is_equal_to(abstract_core::manager::ConfigResponse {
        owner: owner.into_string(),
        account_id: Uint64::from(0u64),
        version_control_address: version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
        is_suspended: false,
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_external() -> AResult {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked(common::OWNER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let deployment = Abstract::deploy_on(chain.clone(), TEST_VERSION.parse().unwrap())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    factory.create_account(
        GovernanceDetails::External {
            governance_address: owner.to_string(),
            governance_type: "some-gov-type".to_string(),
        },
        String::from("first_os"),
        Some(String::from("account_description")),
        Some(String::from("account_link_of_at_least_11_char")),
    )?;

    let account = AbstractAccount::new(chain, Some(0));
    let account_config = account.manager.config()?;

    assert_that!(account_config).is_equal_to(abstract_core::manager::ConfigResponse {
        owner: owner.into_string(),
        account_id: Uint64::from(0u64),
        is_suspended: false,
        version_control_address: version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
    });

    Ok(())
}
