mod common;
use abstract_boot::{AbstractAccount, OsFactoryExecFns, OsFactoryQueryFns, VCQueryFns, *};
use abstract_core::{
    account_factory, objects::gov_type::GovernanceDetails, version_control::AccountBase,
    ABSTRACT_EVENT_NAME,
};
use boot_core::{
    IndexResponse, {instantiate_default_mock_env, ContractInstance},
};
use common::init_abstract_env;
use cosmwasm_std::{Addr, Uint64};
use speculoos::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

#[test]
fn instantiate() -> AResult {
    let _not_owner = Addr::unchecked("not_owner");
    let sender = Addr::unchecked(common::OWNER);
    let (_state, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut account) = init_abstract_env(chain)?;
    deployment.deploy(&mut account)?;

    let factory = deployment.account_factory;
    let factory_config = factory.config()?;
    let expected = account_factory::ConfigResponse {
        owner: sender.into_string(),
        ans_host_contract: deployment.ans_host.address()?.into(),
        version_control_contract: deployment.version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
        subscription_address: None,
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
    let (mut deployment, mut account) = init_abstract_env(chain)?;
    deployment.deploy(&mut account)?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    let os_creation = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        String::from("first_os"),
        Some(String::from("os_description")),
        Some(String::from("os_link_of_at_least_11_char")),
    )?;

    let manager = os_creation.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
    let proxy = os_creation.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;

    let factory_config = factory.config()?;
    let expected = account_factory::ConfigResponse {
        owner: sender.clone().into_string(),
        ans_host_contract: deployment.ans_host.address()?.into(),
        version_control_contract: deployment.version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
        subscription_address: None,
        next_account_id: 1,
    };

    assert_that!(&factory_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_core::version_control::ConfigResponse {
        admin: sender.into_string(),
        factory: factory.address()?.into_string(),
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let os_list = version_control.account_base(0)?;

    assert_that!(&os_list.account_base).is_equal_to(AccountBase {
        manager: Addr::unchecked(manager),
        proxy: Addr::unchecked(proxy),
    });

    Ok(())
}

#[test]
fn create_two_os_s() -> AResult {
    let _not_owner = Addr::unchecked("not_owner");
    let sender = Addr::unchecked(common::OWNER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut account) = init_abstract_env(chain)?;
    deployment.deploy(&mut account)?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    // first account
    let os_1 = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        String::from("first_os"),
        Some(String::from("os_description")),
        Some(String::from("os_link_of_at_least_11_char")),
    )?;
    // second account
    let os_2 = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        String::from("second_os"),
        Some(String::from("os_description")),
        Some(String::from("os_link_of_at_least_11_char")),
    )?;

    let manager1 = os_1.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
    let proxy1 = os_1.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;

    let manager2 = os_2.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
    let proxy2 = os_2.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;

    let factory_config = factory.config()?;
    let expected = account_factory::ConfigResponse {
        owner: sender.clone().into_string(),
        ans_host_contract: deployment.ans_host.address()?.into(),
        version_control_contract: deployment.version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
        subscription_address: None,
        next_account_id: 2,
    };

    assert_that!(&factory_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_core::version_control::ConfigResponse {
        admin: sender.into_string(),
        factory: factory.address()?.into_string(),
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let os_1 = version_control.account_base(0)?.account_base;
    assert_that!(&os_1).is_equal_to(AccountBase {
        manager: Addr::unchecked(manager1),
        proxy: Addr::unchecked(proxy1),
    });

    let os_2 = version_control.account_base(1)?.account_base;
    assert_that!(&os_2).is_equal_to(AccountBase {
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
    let (mut deployment, mut account) = init_abstract_env(chain.clone())?;
    deployment.deploy(&mut account)?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    let os_creation = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: owner.to_string(),
        },
        String::from("first_os"),
        Some(String::from("os_description")),
        Some(String::from("os_link_of_at_least_11_char")),
    )?;

    let manager = os_creation.event_attr_value(ABSTRACT_EVENT_NAME, "manager_address")?;
    let proxy = os_creation.event_attr_value(ABSTRACT_EVENT_NAME, "proxy_address")?;

    let account = version_control.account_base(0)?.account_base;

    let os_1 = AbstractAccount::new(chain, Some(0));
    assert_that!(AccountBase {
        manager: os_1.manager.address()?,
        proxy: os_1.proxy.address()?,
    })
    .is_equal_to(&account);

    assert_that!(AccountBase {
        manager: Addr::unchecked(manager),
        proxy: Addr::unchecked(proxy),
    })
    .is_equal_to(&account);

    let os_config = os_1.manager.config()?;

    assert_that!(os_config).is_equal_to(abstract_core::manager::ConfigResponse {
        owner: owner.into_string(),
        account_id: Uint64::from(0u64),
        version_control_address: version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_external() -> AResult {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked(common::OWNER);
    let (_, chain) = instantiate_default_mock_env(&sender)?;
    let (mut deployment, mut account) = init_abstract_env(chain.clone())?;
    deployment.deploy(&mut account)?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    factory.create_account(
        GovernanceDetails::External {
            governance_address: owner.to_string(),
            governance_type: "some_gov_description".to_string(),
        },
        String::from("first_os"),
        Some(String::from("os_description")),
        Some(String::from("os_link_of_at_least_11_char")),
    )?;

    let account = AbstractAccount::new(chain, Some(0));
    let os_config = account.manager.config()?;

    assert_that!(os_config).is_equal_to(abstract_core::manager::ConfigResponse {
        owner: owner.into_string(),
        account_id: Uint64::from(0u64),
        version_control_address: version_control.address()?.into_string(),
        module_factory_address: deployment.module_factory.address()?.into_string(),
    });

    Ok(())
}
