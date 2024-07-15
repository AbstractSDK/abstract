//! Currently you can run only 1 test at a time: `cargo ct`
//! Otherwise you will have too many requests

use abstract_app::mock::MockInitMsg;
use abstract_framework_clone_testing::common;
use abstract_integration_tests::manager::mock_app::{MockApp, APP_VERSION};
use abstract_interface::{Abstract, AppDeployer, DeployStrategy, VCExecFns};
use abstract_std::{objects::gov_type::GovernanceDetails, PROXY};
use abstract_testing::prelude::*;
use anyhow::Ok;
use cw_orch::{daemon::networks::JUNO_1, prelude::*};
use cw_orch_clone_testing::CloneTesting;
// owner of the abstract infra
const SENDER: &str = "juno1kjzpqv393k4g064xh04j4hwy5d0s03wfvqejga";

fn setup_migrate_allowed_direct_module_registration(
) -> anyhow::Result<(Abstract<CloneTesting>, CloneTesting)> {
    let (deployment, chain) = common::setup(JUNO_1, Addr::unchecked(SENDER))?;
    deployment.migrate_if_version_changed()?;
    deployment
        .version_control
        .update_config(None, None, Some(true))?;
    Ok((deployment, chain))
}

#[test]
fn migrate_infra_success() -> anyhow::Result<()> {
    let (abstr_deployment, _) = common::setup(JUNO_1, Addr::unchecked(SENDER))?;

    let pre_code_id = abstr_deployment.version_control.code_id()?;
    let migrated = abstr_deployment.migrate_if_version_changed()?;
    if migrated {
        assert_ne!(abstr_deployment.version_control.code_id()?, pre_code_id);
    } else {
        // Just so there's something in the log,
        // for the opposite case since this test can be inconsistent
        println!("Nothing to migrate")
    }
    Ok(())
}

#[test]
fn old_account_migrate() -> anyhow::Result<()> {
    let (abstr_deployment, chain) = common::setup(JUNO_1, Addr::unchecked(SENDER))?;

    let old_account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: chain.sender_addr().to_string(),
            })?;

    let migrated = abstr_deployment.migrate_if_version_changed()?;

    if migrated {
        old_account.upgrade(&abstr_deployment)?;
        let info = old_account.manager.module_info(PROXY)?.unwrap();
        assert_eq!(info.version.version, TEST_VERSION)
    } else {
        println!("Nothing to migrate")
    }
    Ok(())
}

#[test]
// FIXME: un-ignore it when possible
#[ignore = "0.23 includes massive ownership revamp which is not compatible with new versions"]
fn old_account_functions() -> anyhow::Result<()> {
    let (abstr_deployment, chain) = common::setup(JUNO_1, Addr::unchecked(SENDER))?;

    let old_account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: chain.sender_addr().to_string(),
            })?;
    let migrated = abstr_deployment.migrate_if_version_changed()?;

    if migrated {
        // Claim namespace
        abstr_deployment
            .version_control
            .claim_namespace(old_account.id()?, "tester".to_owned())?;
        // Allow registration
        abstr_deployment
            .version_control
            .update_config(None, None, Some(true))?;
        // Try to install
        let app = MockApp::new_test(chain.clone());
        MockApp::deploy(&app, APP_VERSION.parse().unwrap(), DeployStrategy::Try)?;
        let res = old_account.install_app(&app, &MockInitMsg {}, None);
        // An old account should be able to install new apps
        assert!(res.is_ok());
    } else {
        println!("Nothing to migrate")
    }
    Ok(())
}

mod manager {
    use abstract_integration_tests::manager::{
        account_install_app, account_move_ownership_to_sub_account,
        create_account_with_installed_module_monetization_and_init_funds,
        create_sub_account_with_modules_installed, install_app_with_proxy_action,
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
    fn install_app_with_proxy_action_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;

        install_app_with_proxy_action(chain)
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
    use abstract_integration_tests::account_factory::create_one_account_with_namespace_fee;

    use super::*;

    #[test]
    fn create_one_account_with_namespace_fee_after_migrate() -> anyhow::Result<()> {
        let (_, chain) = setup_migrate_allowed_direct_module_registration()?;

        create_one_account_with_namespace_fee(chain)
    }
}

mod version_control {

    use abstract_interface::VCQueryFns;

    use super::*;

    #[cosmwasm_schema::cw_serde]
    pub struct Config0_21 {
        pub account_factory_address: Option<Addr>,
        pub allow_direct_module_registration_and_updates: bool,
        pub namespace_registration_fee: Option<Coin>,
    }

    // TODO: remove after 0.22 deployed
    #[test]
    fn version_control0_21_config_migration() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = common::setup(JUNO_1, Addr::unchecked(SENDER))?;

        // Check if not migrated yet
        let vc_version_bytes = chain.wasm_querier().raw_query(
            abstr_deployment.version_control.address()?,
            cw2::CONTRACT.as_slice().to_vec(),
        )?;
        let vc_version: cw2::ContractVersion = from_json(vc_version_bytes)?;
        if vc_version.version != "0.21.0" {
            println!("Vc already migrated, remove this test please");
            return Ok(());
        }

        let old_config: Config0_21 = abstr_deployment
            .version_control
            .query(&abstract_std::version_control::QueryMsg::Config {})?;

        abstr_deployment.migrate_if_version_changed()?;

        let config = abstr_deployment.version_control.config()?;
        assert_eq!(
            old_config.account_factory_address,
            config.account_factory_address
        );
        assert_eq!(
            old_config.allow_direct_module_registration_and_updates,
            config.security_disabled
        );
        assert_eq!(
            old_config.namespace_registration_fee,
            config.namespace_registration_fee
        );

        Ok(())
    }
}
