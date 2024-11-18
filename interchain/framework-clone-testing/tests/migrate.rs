//! Currently you can run only 1 test at a time: `cargo ct`
//! Otherwise you will have too many requests

use abstract_app::mock::MockInitMsg;
use abstract_framework_clone_testing::common;
use abstract_integration_tests::account::mock_app::{MockApp, APP_VERSION};
use abstract_interface::{Abstract, AccountI, AppDeployer, DeployStrategy, RegistryExecFns};
use abstract_std::objects::gov_type::GovernanceDetails;
use abstract_testing::prelude::*;
use anyhow::Ok;
use cw_orch::{daemon::networks::PION_1, prelude::*};
use cw_orch_clone_testing::CloneTesting;

fn setup_migrate_allowed_direct_module_registration(
) -> anyhow::Result<(Abstract<CloneTesting>, CloneTesting)> {
    let (deployment, chain) = common::setup(PION_1)?;
    deployment.migrate_if_version_changed()?;
    deployment.registry.update_config(None, Some(true))?;
    Ok((deployment, chain))
}

#[test]
fn migrate_infra_success() -> anyhow::Result<()> {
    let (abstr_deployment, _) = common::setup(PION_1)?;

    let pre_code_id = abstr_deployment.registry.code_id()?;
    let migrated = abstr_deployment.migrate_if_version_changed()?;
    if migrated {
        assert_ne!(abstr_deployment.registry.code_id()?, pre_code_id);
    } else {
        // Just so there's something in the log,
        // for the opposite case since this test can be inconsistent
        println!("Nothing to migrate")
    }
    Ok(())
}

#[test]
fn old_account_migrate() -> anyhow::Result<()> {
    let (abstr_deployment, chain) = common::setup(PION_1)?;

    let old_account = AccountI::create_default_account(
        &abstr_deployment,
        GovernanceDetails::Monarchy {
            monarch: chain.sender_addr().to_string(),
        },
    )?;

    let migrated = abstr_deployment.migrate_if_version_changed()?;

    if migrated {
        old_account.upgrade_account(&abstr_deployment)?;
        let info = old_account.item_query(cw2::CONTRACT)?;
        assert_eq!(info.version, TEST_VERSION)
    } else {
        println!("Nothing to migrate")
    }
    Ok(())
}

#[test]
fn old_account_functions() -> anyhow::Result<()> {
    let (abstr_deployment, chain) = common::setup(PION_1)?;

    let old_account = AccountI::create_default_account(
        &abstr_deployment,
        GovernanceDetails::Monarchy {
            monarch: chain.sender_addr().to_string(),
        },
    )?;
    let migrated = abstr_deployment.migrate_if_version_changed()?;

    if migrated {
        // Claim namespace
        abstr_deployment
            .registry
            .claim_namespace(old_account.id()?, "tester".to_owned())?;
        // Allow registration
        abstr_deployment.registry.update_config(None, Some(true))?;
        // Try to install
        let app = MockApp::new_test(chain.clone());
        MockApp::deploy(&app, APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
        let res = old_account.install_app(&app, &MockInitMsg {}, &[]);
        // An old account should be able to install new apps
        assert!(res.is_ok());
    } else {
        println!("Nothing to migrate")
    }
    Ok(())
}

mod account {
    use abstract_integration_tests::account::{
        account_install_app, account_move_ownership_to_sub_account,
        create_account_with_installed_module_monetization_and_init_funds,
        create_sub_account_with_modules_installed, install_app_with_account_action,
        installing_one_adapter_with_fee_should_succeed, uninstall_modules,
        update_adapter_with_authorized_addrs, with_response_data,
    };

    use super::*;

    #[test]
    fn install_app_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;
        account_install_app(chain)
    }

    #[test]
    fn create_sub_account_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;
        create_sub_account_with_modules_installed(chain)
    }

    #[test]
    fn create_account_with_installed_module_monetization_and_init_funds_after_migrate(
    ) -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;
        create_account_with_installed_module_monetization_and_init_funds(chain, ("coin1", "coin2"))
    }

    #[test]
    fn install_app_with_account_action_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;

        install_app_with_account_action(chain)
    }

    #[test]
    fn update_adapter_with_authorized_addrs_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;

        let authorizee = chain.init_account();
        update_adapter_with_authorized_addrs(chain, authorizee)
    }

    #[test]
    fn uninstall_modules_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;

        uninstall_modules(chain)
    }

    #[test]
    fn installing_one_adapter_with_fee_should_succeed_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;

        installing_one_adapter_with_fee_should_succeed(chain)
    }

    #[test]
    fn with_response_data_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;

        with_response_data(chain)
    }

    #[test]
    fn account_move_ownership_to_sub_account_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;

        account_move_ownership_to_sub_account(chain)
    }
}

mod account_factory {

    use abstract_integration_tests::create::create_one_account_with_namespace_fee;

    use super::*;

    #[test]
    fn create_one_account_with_namespace_fee_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;

        create_one_account_with_namespace_fee(chain)
    }
}

mod from_xion {
    use super::*;
    use abstract_interface::{AccountExecFns, AccountI};
    use abstract_std::{account::MigrateMsg, IBC_CLIENT};
    use networks::XION_TESTNET_1;

    pub const XION_ACCOUNT: &str =
        "xion1c8lhvl6hun9jfd7rvpjyprnf3c70utlvwvdxk94s43t5qaqcze9q6qz0y4";

    #[test]
    fn migrate_from_xion_account() -> anyhow::Result<()> {
        let (deployment, chain) = common::setup(XION_TESTNET_1)?;

        // We need to register the new code id
        deployment.migrate_if_version_changed()?;

        // This is a XION user action
        let addr_contract = Addr::unchecked(XION_ACCOUNT);
        let account = AccountI::new("account-xion", chain);
        account.set_address(&addr_contract);

        account
            .call_as(&addr_contract)
            .migrate(&MigrateMsg {}, deployment.account_code_id()?)?;

        account
            .update_info(None, None, Some("brand new abstract account".to_string()))
            .unwrap_err();

        account.call_as(&addr_contract).update_info(
            None,
            None,
            Some("brand new abstract account".to_string()),
        )?;

        assert!(account.is_module_installed(IBC_CLIENT)?);
        Ok(())
    }
}
