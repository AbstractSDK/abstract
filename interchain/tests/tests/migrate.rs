mod common;

use abstract_core::objects::AccountId;
use abstract_interface::Abstract;
use abstract_interface::*;
use anyhow::Ok;
use cosmwasm_std::Addr;
use cw20::Cw20QueryMsg;
use cw_orch::daemon::networks::JUNO_1;
use cw_orch::prelude::*;
use cw_orch_fork_mock::ForkMock;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::runtime::Runtime;

use cosmwasm_std::Empty;

const VERSION: &str = env!("CARGO_PKG_VERSION");
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
fn setup() -> anyhow::Result<(Abstract<ForkMock>, Addr, ForkMock)> {
    env_logger::init();
    let sender = Addr::unchecked(SENDER);
    // Run migration tests against Juno mainnet
    let mut app = ForkMock::new(rt(), JUNO_1)?;
    app.set_sender(sender.clone());

    let abstr_deployment = Abstract::load_from(app.clone())?;
    Ok((abstr_deployment, sender, app))
}

#[test]
fn migrate_infra_success() -> anyhow::Result<()> {
    let (abstr_deployment, sender, _) = setup()?;
    
    let pre_code_id = abstr_deployment.version_control.code_id()?;
    abstr_deployment.migrate_if_needed()?;
    assert_ne!(abstr_deployment.version_control.code_id()?, pre_code_id);
    Ok(())
}

fn install_app_after_migrate() -> anyhow::Result<()> {
    let (abstr_deployment, sender, app) = setup()?;
    abstr_deployment.migrate_if_needed()?;
    abstract_integration_tests::manager::account_install_app(app.clone(), sender)
}

fn create_sub_account_after_migrate() -> anyhow::Result<()> {
    let (abstr_deployment, sender, app) = setup()?;
    abstr_deployment.migrate_if_needed()?;
    abstract_integration_tests::manager::create_sub_account_with_modules_installed(
        app.clone(),
        sender,
    )
}
