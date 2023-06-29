//! Deploys Abstract and the App module to a local Junod instance. See how to spin up a local chain here: https://docs.junonetwork.io/developer-guides/junod-local-dev-setup
//!
//! Ensure the local juno is running before executing this script.
//!
//! # Run
//!
//! `cargo run --example test-local`

use abstract_core::{app::BaseInstantiateMsg, objects::gov_type::GovernanceDetails};
use cosmwasm_std::Empty;
use cw_orch::{
    anyhow,
    deploy::Deploy,
    prelude::{networks::LOCAL_JUNO, ContractInstance, Daemon, TxHandler},
    tokio::runtime::Runtime,
};
use abstract_interface::{Abstract, AppDeployer, VCExecFns};
use app::{
    contract::{APP_ID, APP_VERSION},
    msg::AppInstantiateMsg,
    AppInterface,
};
use semver::Version;
use speculoos::{assert_that, prelude::BooleanAssertions};

const LOCAL_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let version: Version = APP_VERSION.parse().unwrap();
    let runtime = Runtime::new()?;

    let daemon = Daemon::builder()
        .chain(LOCAL_JUNO)
        .mnemonic(LOCAL_MNEMONIC)
        .handle(runtime.handle())
        .build()
        .unwrap();

    let abstract_deployment = Abstract::deploy_on(daemon.clone(), Empty {})?;

    let app = AppInterface::new(APP_ID, daemon.clone());

    // Create account
    let account = abstract_deployment.account_factory.create_default_account(
        GovernanceDetails::Monarchy {
            monarch: daemon.sender().into_string(),
        },
    )?;

    // Claim namespace
    abstract_deployment.version_control.claim_namespaces(
        account.id()?,
        vec!["my-namespace".to_owned()],
    )?;

    // Deploy
    app.deploy(version)?;

    // Install app
    account.install_module(
        APP_ID,
        &app::msg::InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: abstract_deployment.ans_host.addr_str()?,
            },
            module: AppInstantiateMsg {},
        },
        None,
    )?;

    assert_that!(account.manager.is_module_installed(APP_ID).unwrap()).is_true();
    Ok(())
}
