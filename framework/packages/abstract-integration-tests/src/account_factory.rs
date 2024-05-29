use abstract_interface::{Abstract, AccountDetails, AccountFactoryExecFns, VCExecFns, VCQueryFns};
use abstract_sdk::cw_helpers::Clearable;
use abstract_std::{
    objects::{gov_type::GovernanceDetails, namespace::Namespace},
    version_control::{AccountBase, NamespaceInfo, NamespaceResponse},
};
use cosmwasm_std::{coin, Uint128};
use cw_orch::{environment::MutCwEnv, prelude::*};

use crate::AResult;

pub fn create_one_account_with_namespace_fee<T: MutCwEnv>(mut chain: T) -> AResult {
    let deployment = Abstract::load_from(chain.clone())?;
    let sender = chain.sender();
    let bs_profile = Some(String::from("the-monk-on-iron-mountain-2"));
    let factory = &deployment.account_factory;
    let version_control = &deployment.version_control;

    // Update namespace fee
    let namespace_fee = coin(10, "token");
    let profile_fee = coin(1000, "ubtsg");
    chain
        .set_balance(&sender, vec![namespace_fee.clone(), profile_fee.clone()])
        .unwrap();
    version_control.update_config(None, Some(Clearable::Set(namespace_fee.clone())), None)?;

    factory.update_config(
        None,
        None,
        None,
        None,
        None,
        Some(Uint128::new(1000u128)),
        None,
        None,
    )?;

    let namespace_to_claim = "namespace-to-claim";

    let err = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_account"),
        None,
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

    let err = factory.create_account(
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        vec![],
        String::from("first_account"),
        None,
        None,
        Some(String::from("the-monk-on-iron-mountain")),
        None,
        Some(String::from("https://account_link_of_at_least_11_char")),
        None,
        // Profile creation fee not covered
        &[],
    );
    assert!(err
        .unwrap_err()
        // Error type is inside contract, not the package
        .root()
        .to_string()
        .contains("Invalid profile fee payment sent."));

    // Now cover account creation fee
    let account = factory.create_new_account(
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(namespace_to_claim.to_string()),
            base_asset: None,
            install_modules: vec![],
            account_id: None,
            bs_profile: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        Some(&[namespace_fee]),
    )?;

    let manager_addr = account.manager.address()?;
    let proxy_addr = account.proxy.address()?;

    // We need to check if the namespace is associated with this account
    let namespace = version_control.namespace(Namespace::new(namespace_to_claim)?)?;

    assert_eq!(
        namespace,
        NamespaceResponse::Claimed(NamespaceInfo {
            account_id: account.id()?,
            account_base: AccountBase {
                manager: manager_addr,
                proxy: proxy_addr,
            }
        })
    );
    // Now cover profile creation fee
    factory.create_new_account(
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: None,
            base_asset: None,
            install_modules: vec![],
            account_id: None,
            bs_profile,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        Some(&[profile_fee]),
    )?;

    Ok(())
}
