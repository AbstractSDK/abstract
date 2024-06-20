//! Currently you can run only 1 test at a time: `cargo ct`
//! Otherwise you will have too many requests

use abstract_framework_clone_testing::common;
use abstract_interface::{Abstract, AbstractAccount, AccountFactoryQueryFns, ManagerQueryFns};
use abstract_std::{objects::AccountId, PROXY};
use anyhow::{bail, Ok};
use cw_orch::{daemon::networks::JUNO_1, prelude::*};
use cw_orch_clone_testing::CloneTesting;
use cw_semver::Version;
// owner of the abstract infra
const SENDER: &str = "juno1kjzpqv393k4g064xh04j4hwy5d0s03wfvqejga";

fn find_old_account() -> anyhow::Result<(CloneTesting, u32, String)> {
    let (abstr_deployment, chain) = common::setup(JUNO_1, Addr::unchecked(SENDER))?;

    // List accounts
    let max_account_id = abstr_deployment
        .account_factory
        .config()?
        .local_account_sequence;

    let (oldest_account_id, owner, _) = (0..max_account_id.min(100))
        .flat_map(|account_id| {
            // Create the account helper
            let account = AbstractAccount::new(&abstr_deployment, AccountId::local(account_id));
            if let Some(owner) = account.manager.ownership()?.owner {
                let proxy_version = Version::parse(
                    &account
                        .manager
                        .module_versions(vec![PROXY.to_string()])?
                        .versions[0]
                        .version,
                )?;

                Ok((account_id, owner, proxy_version))
            } else {
                bail!("No owner for this account")
            }
        })
        .min_by(|(_, _, v1), (_, _, v2)| v1.cmp(v2))
        .unwrap();

    // List the manager and proxy versions

    Ok((chain, oldest_account_id, owner))
}

#[test]
fn upgrade_account_iteratively() -> anyhow::Result<()> {
    let (chain, account_id, owner) = find_old_account()?;

    let abstr_deployment = Abstract::load_from(chain.call_as(&Addr::unchecked(owner)).clone())?;
    let account = AbstractAccount::new(&abstr_deployment, AccountId::local(account_id));

    account.upgrade()?;

    Ok(())
}
