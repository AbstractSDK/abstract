use abstract_interface::{
    Abstract, AccountDetails, AccountI, VCExecFns, VCQueryFns,
};
use abstract_sdk::cw_helpers::Clearable;
use abstract_std::{
    objects::{gov_type::GovernanceDetails, namespace::Namespace},
    version_control::{Account, NamespaceInfo, NamespaceResponse},
};
use cosmwasm_std::coin;
use cw_orch::{environment::MutCwEnv, prelude::*};

use crate::AResult;

pub fn create_one_account_with_namespace_fee<T: MutCwEnv>(mut chain: T) -> AResult {
    let deployment = Abstract::load_from(chain.clone())?;
    let sender = chain.sender_addr();

    let version_control = &deployment.version_control;

    // Update namespace fee
    let namespace_fee = coin(10, "token");
    chain
        .set_balance(&sender, vec![namespace_fee.clone()])
        .unwrap();
    version_control.update_config(Some(Clearable::Set(namespace_fee.clone())), None)?;

    let namespace_to_claim = "namespace-to-claim";

    let err = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(namespace_to_claim.to_string()),
            install_modules: vec![],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
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
    let account = AccountI::create(
        &deployment,
        AccountDetails {
            name: String::from("first_account"),
            description: Some(String::from("account_description")),
            link: Some(String::from("https://account_link_of_at_least_11_char")),
            namespace: Some(namespace_to_claim.to_string()),
            install_modules: vec![],
            account_id: None,
        },
        GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        },
        &[namespace_fee],
    )?;

    let account_addr = account.address()?;

    // We need to check if the namespace is associated with this account
    let namespace = version_control.namespace(Namespace::new(namespace_to_claim)?)?;

    assert_eq!(
        namespace,
        NamespaceResponse::Claimed(NamespaceInfo {
            account_id: account.id()?,
            account_base: Account::new(account_addr.clone()),
        })
    );

    Ok(())
}
