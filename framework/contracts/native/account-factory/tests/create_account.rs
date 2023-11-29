mod common;

use abstract_core::ans_host::ExecuteMsgFns;
use abstract_core::objects::account::AccountTrace;
use abstract_core::objects::namespace::Namespace;
use abstract_core::objects::AccountId;
use abstract_core::objects::AssetEntry;
use abstract_core::proxy::BaseAssetResponse;
use abstract_core::version_control::NamespaceResponse;
use abstract_core::PROXY;
use abstract_core::{
    account_factory, objects::gov_type::GovernanceDetails, version_control::AccountBase,
    ABSTRACT_EVENT_TYPE,
};
use abstract_interface::{
    AbstractAccount, AccountFactoryExecFns, AccountFactoryQueryFns, VCQueryFns, *,
};
use abstract_testing::prelude::*;
use cosmwasm_std::coin;
use cosmwasm_std::Addr;
use cw_asset::{AssetInfo, AssetInfoBase};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::Mock;
use cw_orch::prelude::*;
use speculoos::prelude::*;

type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

#[test]
fn instantiate() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let factory = deployment.account_factory;
    let factory_config = factory.config()?;
    let expected = account_factory::ConfigResponse {
        ans_host_contract: deployment.ans_host.address()?,
        version_control_contract: deployment.version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        local_account_sequence: 1,
        ibc_host: Some(deployment.ibc.host.address()?),
    };

    assert_that!(&factory_config).is_equal_to(&expected);
    Ok(())
}

#[test]
fn create_one_account() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    let account_creation = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_account"),
        None,
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;

    let manager = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "manager_address")?;
    let proxy = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "proxy_address")?;

    let factory_config = factory.config()?;
    let expected = account_factory::ConfigResponse {
        ans_host_contract: deployment.ans_host.address()?,
        version_control_contract: deployment.version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        local_account_sequence: 2,
        ibc_host: Some(deployment.ibc.host.address()?),
    };

    assert_that!(&factory_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_core::version_control::ConfigResponse {
        factory: factory.address()?,
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let account_list = version_control.account_base(TEST_ACCOUNT_ID)?;

    assert_that!(&account_list.account_base).is_equal_to(AccountBase {
        manager: Addr::unchecked(manager),
        proxy: Addr::unchecked(proxy),
    });

    Ok(())
}

#[test]
fn create_two_account_s() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    // first account
    let account_1 = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_os"),
        None,
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;
    // second account
    let account_2 = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("second_os"),
        None,
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;

    let manager1 = account_1.event_attr_value(ABSTRACT_EVENT_TYPE, "manager_address")?;
    let proxy1 = account_1.event_attr_value(ABSTRACT_EVENT_TYPE, "proxy_address")?;
    let account_1_id = TEST_ACCOUNT_ID;

    let manager2 = account_2.event_attr_value(ABSTRACT_EVENT_TYPE, "manager_address")?;
    let proxy2 = account_2.event_attr_value(ABSTRACT_EVENT_TYPE, "proxy_address")?;
    let account_2_id = AccountId::new(TEST_ACCOUNT_ID.seq() + 1, AccountTrace::Local)?;

    let factory_config = factory.config()?;
    let expected = account_factory::ConfigResponse {
        ans_host_contract: deployment.ans_host.address()?,
        version_control_contract: deployment.version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        // we created two accounts
        local_account_sequence: account_2_id.seq() + 1,
        ibc_host: Some(deployment.ibc.host.address()?),
    };

    assert_that!(&factory_config).is_equal_to(&expected);

    let vc_config = version_control.config()?;
    let expected = abstract_core::version_control::ConfigResponse {
        factory: factory.address()?,
    };

    assert_that!(&vc_config).is_equal_to(&expected);

    let account_1 = version_control.account_base(account_1_id)?.account_base;
    assert_that!(&account_1).is_equal_to(AccountBase {
        manager: Addr::unchecked(manager1),
        proxy: Addr::unchecked(proxy1),
    });

    let account_2 = version_control.account_base(account_2_id)?.account_base;
    assert_that!(&account_2).is_equal_to(AccountBase {
        manager: Addr::unchecked(manager2),
        proxy: Addr::unchecked(proxy2),
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_monarchy() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    let account_creation = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: OWNER.to_string(),
        },
        vec![],
        String::from("first_os"),
        None,
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;

    let manager = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "manager_address")?;
    let proxy = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "proxy_address")?;

    let account = version_control.account_base(TEST_ACCOUNT_ID)?.account_base;

    let account_1 = AbstractAccount::new(&deployment, Some(TEST_ACCOUNT_ID));
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
        account_id: TEST_ACCOUNT_ID,
        version_control_address: version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
        is_suspended: false,
    });

    Ok(())
}

#[test]
fn sender_is_not_admin_external() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain, sender.to_string())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;
    factory.create_account(
        GovernanceDetails::External {
            governance_address: OWNER.to_string(),
            governance_type: "some-gov-type".to_string(),
        },
        vec![],
        String::from("first_os"),
        None,
        None,
        Some(String::from("account_description")),
        Some(String::from("http://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;

    let account = AbstractAccount::new(&deployment, Some(TEST_ACCOUNT_ID));
    let account_config = account.manager.config()?;

    assert_that!(account_config).is_equal_to(abstract_core::manager::ConfigResponse {
        account_id: TEST_ACCOUNT_ID,
        is_suspended: false,
        version_control_address: version_control.address()?,
        module_factory_address: deployment.module_factory.address()?,
    });

    Ok(())
}

#[test]
fn create_one_account_with_base_asset() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let factory = &deployment.account_factory;
    let ans_host = &deployment.ans_host;

    // Register the "juno", test asset for usage with the account
    let asset_name = "juno";
    let asset = AssetInfoBase::Native("ujuno".to_string());
    let checked_asset = AssetInfo::Native("ujuno".to_string());
    ans_host.update_asset_addresses(vec![(asset_name.to_string(), asset)], vec![])?;

    let account_creation = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_account"),
        None,
        Some(AssetEntry::new(asset_name)),
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        None,
        &[],
    )?;

    let proxy_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "proxy_address")?;

    let proxy = Proxy::new(PROXY, chain.clone());
    proxy.set_address(&Addr::unchecked(proxy_addr));

    let base_asset = proxy.base_asset()?;

    assert_that!(&base_asset).is_equal_to(&BaseAssetResponse {
        base_asset: checked_asset,
    });

    Ok(())
}

#[test]
fn create_one_account_with_namespace() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;

    let namespace_to_claim = "namespace-to-claim";

    let account_creation = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_account"),
        None,
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        Some(namespace_to_claim.to_string()),
        &[],
    )?;

    let manager_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "manager_address")?;
    let proxy_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "proxy_address")?;

    // We need to check if the namespace is associated with this account
    let namespace = version_control.namespace(Namespace::new(namespace_to_claim)?)?;

    assert_that!(&namespace).is_equal_to(&NamespaceResponse {
        account_id: TEST_ACCOUNT_ID,
        account_base: AccountBase {
            manager: Addr::unchecked(manager_addr),
            proxy: Addr::unchecked(proxy_addr),
        },
    });

    Ok(())
}

#[test]
fn create_one_account_with_namespace_fee() -> AResult {
    let sender = Addr::unchecked(OWNER);
    let chain = Mock::new(&sender);
    let deployment = Abstract::deploy_on(chain.clone(), sender.to_string())?;

    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;

    // Update namespace fee
    let namespace_fee = coin(10, "token");
    chain.set_balance(&sender, vec![namespace_fee.clone()])?;
    version_control.update_config(None, Some(namespace_fee.clone()))?;

    let namespace_to_claim = "namespace-to-claim";

    let err = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_account"),
        None,
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        Some(namespace_to_claim.to_string()),
        // Account creation fee not covered
        &[],
    );
    assert!(err
        .unwrap_err()
        // Error type is inside contract, not the package
        .root()
        .to_string()
        .contains("Invalid fee payment sent."));

    // Now cover account creation fee
    let account_creation = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_account"),
        None,
        None,
        Some(String::from("account_description")),
        Some(String::from("https://account_link_of_at_least_11_char")),
        Some(namespace_to_claim.to_string()),
        &[namespace_fee],
    )?;

    let manager_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "manager_address")?;
    let proxy_addr = account_creation.event_attr_value(ABSTRACT_EVENT_TYPE, "proxy_address")?;

    // We need to check if the namespace is associated with this account
    let namespace = version_control.namespace(Namespace::new(namespace_to_claim)?)?;

    assert_that!(&namespace).is_equal_to(&NamespaceResponse {
        account_id: TEST_ACCOUNT_ID,
        account_base: AccountBase {
            manager: Addr::unchecked(manager_addr),
            proxy: Addr::unchecked(proxy_addr),
        },
    });

    Ok(())
}
