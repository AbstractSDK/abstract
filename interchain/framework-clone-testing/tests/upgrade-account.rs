//! Currently you can run only 1 test at a time: `cargo ct`
//! Otherwise you will have too many requests

use abstract_framework_clone_testing::common;
use abstract_interface::{Abstract, AccountDetails, AccountI, AccountQueryFns, RegistryQueryFns};
use abstract_std::objects::AccountId;
use abstract_unit_test_utils::TEST_VERSION;
use anyhow::{bail, Ok};
use cw_orch::{daemon::networks::JUNO_1, prelude::*};
use cw_orch_clone_testing::CloneTesting;
use semver::Version;

fn find_old_account() -> anyhow::Result<(CloneTesting, u32, Addr)> {
    let (abstr_deployment, chain) = common::setup(JUNO_1)?;

    abstr_deployment.migrate_if_version_changed()?;
    // List accounts
    let max_account_id = abstr_deployment.registry.config()?.local_account_sequence;

    let (oldest_account_id, owner, _) = (0..max_account_id.min(100))
        .flat_map(|account_id| {
            // Create the account helper
            let account = AccountI::load_from(&abstr_deployment, AccountId::local(account_id))?;
            if let Some(owner) = account.top_level_owner().ok() {
                let ver = account.item_query(cw2::CONTRACT)?;
                let account_version = Version::parse(&ver.version)?;

                Ok((account_id, owner.address, account_version))
            } else {
                bail!("No owner for this account")
            }
        })
        .min_by(|(_, _, v1), (_, _, v2)| v1.cmp(v2))
        .unwrap();

    Ok((chain, oldest_account_id, owner))
}

#[test]
fn upgrade_account_iteratively() -> anyhow::Result<()> {
    let (chain, account_id, owner) = find_old_account()?;

    let abstr_deployment = Abstract::load_from(chain.call_as(&owner).clone())?;
    let account = AccountI::load_from(&abstr_deployment, AccountId::local(account_id))?;

    account.upgrade_account(&abstr_deployment)?;
    let info_account = account.item_query(cw2::CONTRACT)?;
    assert_eq!(info_account.version, TEST_VERSION);

    Ok(())
}

#[test]
fn upgrade_accounts_and_sub_accounts() -> anyhow::Result<()> {
    let (abstr_deployment, chain) = common::setup(JUNO_1)?;

    let account = AccountI::create_default_account(
        &abstr_deployment,
        abstract_client::GovernanceDetails::Monarchy {
            monarch: chain.sender_addr().to_string(),
        },
    )?;
    let sub_account = account.create_and_return_sub_account(
        AccountDetails {
            name: "sub_account_one".to_string(),
            ..Default::default()
        },
        &[],
    )?;
    let sub_sub_account = sub_account.create_and_return_sub_account(
        AccountDetails {
            name: "sub_account_two".to_string(),
            ..Default::default()
        },
        &[],
    )?;

    abstr_deployment.migrate_if_version_changed()?;

    account.upgrade_account(&abstr_deployment)?;
    let info_account = account.item_query(cw2::CONTRACT)?;
    assert_eq!(info_account.version, TEST_VERSION);

    let info_sub_account = sub_account.item_query(cw2::CONTRACT)?;
    assert_eq!(info_sub_account.version, TEST_VERSION);

    let info_sub_sub_account = sub_sub_account.item_query(cw2::CONTRACT)?;
    assert_eq!(info_sub_sub_account.version, TEST_VERSION);

    Ok(())
}
