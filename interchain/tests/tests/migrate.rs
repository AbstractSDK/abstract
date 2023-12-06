//! Currently you can run only 1 test at a time: `cargo test -- --test-threads=1`
//! Otherwise you will have too many requests

mod common;

use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_core::objects::module::ModuleInfo;
use abstract_core::{ABSTRACT_EVENT_TYPE, MANAGER, PROXY};
use abstract_interface::{Abstract, AbstractAccount, ManagerExecFns};
use abstract_testing::prelude::*;
use anyhow::Ok;
use cosmwasm_std::{to_json_binary, Addr};
use cw_orch::daemon::networks::JUNO_1;
use cw_orch::prelude::*;
use cw_orch_fork_mock::ForkMock;

// owner of the abstract infra
const SENDER: &str = "juno1kjzpqv393k4g064xh04j4hwy5d0s03wfvqejga";

/// Returns a shared tokio runtime for all tests
fn rt() -> &'static tokio::runtime::Runtime {
    lazy_static::lazy_static! {
        static ref RT: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Should create a tokio runtime");
    }
    &RT
}

/// Sets up the forkmock for Juno mainnet.
/// Returns the abstract deployment and sender (=mainnet admin)
fn setup() -> anyhow::Result<(Abstract<ForkMock>, ForkMock)> {
    let _ = env_logger::builder().is_test(true).try_init();
    let sender = Addr::unchecked(SENDER);
    // Run migration tests against Juno mainnet
    let mut app = ForkMock::new(rt(), JUNO_1)?;
    app.set_sender(sender.clone());

    let abstr_deployment = Abstract::load_from(app.clone())?;
    Ok((abstr_deployment, app))
}

#[test]
fn migrate_infra_success() -> anyhow::Result<()> {
    let (abstr_deployment, _) = setup()?;

    let pre_code_id = abstr_deployment.version_control.code_id()?;
    let migrated = abstr_deployment.migrate_if_needed()?;
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
    let (abstr_deployment, chain) = setup()?;

    // Old message had no account_id field, need something to serialize
    #[cosmwasm_schema::cw_serde]
    enum MockAccountFactoryExecuteMsg {
        CreateAccount {
            name: String,
            governance: GovernanceDetails<String>,
            install_modules: Vec<Empty>,
        },
    }

    let account_factory_address = abstr_deployment.account_factory.address()?;
    let result = chain.execute(
        &MockAccountFactoryExecuteMsg::CreateAccount {
            name: "Default name".to_owned(),
            governance: GovernanceDetails::Monarchy {
                monarch: chain.sender().to_string(),
            },
            install_modules: vec![],
        },
        &[],
        &account_factory_address,
    )?;

    let manager_address =
        Addr::unchecked(result.event_attr_value(ABSTRACT_EVENT_TYPE, "manager_address")?);
    let res: abstract_core::manager::ConfigResponse = chain.query(
        &abstract_core::manager::QueryMsg::Config {},
        &manager_address,
    )?;

    let migrated = abstr_deployment.migrate_if_needed()?;

    if migrated {
        let old_account = AbstractAccount::new(&abstr_deployment, res.account_id);

        let account_migrate_modules = vec![
            (
                ModuleInfo::from_id_latest(MANAGER)?,
                Some(to_json_binary(&abstract_core::manager::MigrateMsg {})?),
            ),
            (
                ModuleInfo::from_id_latest(PROXY)?,
                Some(to_json_binary(&abstract_core::proxy::MigrateMsg {})?),
            ),
        ];
        old_account.manager.upgrade(account_migrate_modules)?;
        let info = old_account.manager.module_info(PROXY)?.unwrap();
        assert_eq!(info.version.version, TEST_VERSION)
    } else {
        println!("Nothing to migrate")
    }
    Ok(())
}

mod manager {
    use super::*;

    use abstract_integration_tests::manager::{
        account_install_app, account_move_ownership_to_sub_account,
        create_account_with_installed_module_monetization_and_init_funds,
        create_sub_account_with_modules_installed, install_app_with_proxy_action,
        installing_one_adapter_with_fee_should_succeed, uninstall_modules,
        update_adapter_with_authorized_addrs, with_response_data,
    };

    #[test]
    fn install_app_after_migrate() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;
        account_install_app(chain)
    }

    #[test]
    fn create_sub_account_after_migrate() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;
        create_sub_account_with_modules_installed(chain)
    }

    #[test]
    fn create_account_with_installed_module_monetization_and_init_funds_after_migrate(
    ) -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;
        create_account_with_installed_module_monetization_and_init_funds(chain, ("coin1", "coin2"))
    }

    #[test]
    fn install_app_with_proxy_action_after_migrate() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;
        install_app_with_proxy_action(chain)
    }

    #[test]
    fn update_adapter_with_authorized_addrs_after_migrate() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;

        let authorizee = chain.init_account();
        update_adapter_with_authorized_addrs(chain, authorizee)
    }

    #[test]
    fn uninstall_modules_after_migrate() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;
        uninstall_modules(chain)
    }

    #[test]
    fn installing_one_adapter_with_fee_should_succeed_after_migrate() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;
        installing_one_adapter_with_fee_should_succeed(chain)
    }

    #[test]
    fn with_response_data_after_migrate() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;
        with_response_data(chain)
    }

    #[test]
    fn account_move_ownership_to_sub_account_after_migrate() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;
        account_move_ownership_to_sub_account(chain)
    }
}

mod account_factory {
    use super::*;
    use abstract_integration_tests::account_factory::create_one_account_with_namespace_fee;

    #[test]
    fn create_one_account_with_namespace_fee_after_migrate() -> anyhow::Result<()> {
        let (abstr_deployment, chain) = setup()?;
        abstr_deployment.migrate_if_needed()?;
        create_one_account_with_namespace_fee(chain)
    }
}
