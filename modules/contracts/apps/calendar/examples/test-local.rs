//! Deploys Abstract and the App module to a local Junod instance. See how to spin up a local chain here: https://docs.junonetwork.io/developer-guides/junod-local-dev-setup
//!
//! Ensure the local juno is running before executing this script.
//!
//! # Run
//!
//! `cargo run --example test-local`

use abstract_app::std::objects::{gov_type::GovernanceDetails, AssetEntry};
use abstract_interface::VCExecFns;
use abstract_interface::{Abstract, AppDeployer, DeployStrategy};
use calendar_app::{
    contract::{APP_ID, APP_VERSION},
    msg::{CalendarInstantiateMsg, Time},
    CalendarAppInterface,
};
use cosmwasm_std::Uint128;
use cw_orch::{
    anyhow,
    prelude::{networks::LOCAL_JUNO, Daemon, Deploy, TxHandler},
    tokio::runtime::Runtime,
};
use semver::Version;
use speculoos::{assert_that, prelude::BooleanAssertions};

// From https://github.com/CosmosContracts/juno/blob/32568dba828ff7783aea8cb5bb4b8b5832888255/docker/test-user.env#L2
const LOCAL_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let version: Version = APP_VERSION.parse().unwrap();
    let runtime = Runtime::new()?;

    let daemon = Daemon::builder(LOCAL_JUNO)
        .mnemonic(LOCAL_MNEMONIC)
        .handle(runtime.handle())
        .build()
        .unwrap();
    // Deploy abstract locally
    let abstract_deployment =
        Abstract::deploy_on(daemon.clone(), daemon.sender_addr().to_string())?;

    let app = CalendarAppInterface::new(APP_ID, daemon.clone());

    // Create account
    let account = abstract_deployment.account_factory.create_default_account(
        GovernanceDetails::Monarchy {
            monarch: daemon.sender_addr().into_string(),
        },
    )?;

    // Claim namespace
    abstract_deployment
        .version_control
        .claim_namespace(account.id()?, "my-namespace".to_owned())?;

    // Deploy
    app.deploy(version, DeployStrategy::Try)?;

    // Install app
    account.install_app(
        &app,
        &CalendarInstantiateMsg {
            price_per_minute: Uint128::zero(),
            denom: AssetEntry::from("juno>ujunox"),
            utc_offset: 0,
            start_time: Time { hour: 9, minute: 0 },
            end_time: Time {
                hour: 17,
                minute: 0,
            },
        },
        None,
    )?;

    assert_that!(account.is_module_installed(APP_ID).unwrap()).is_true();
    Ok(())
}
