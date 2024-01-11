//! Deploys Abstract and the App module to a local Junod instance. See how to spin up a local chain here: https://docs.junonetwork.io/developer-guides/junod-local-dev-setup
//! You can also start a juno container by running `just juno-local`.
//!
//! Ensure the local juno is running before executing this script.
//! Also make sure port 9090 is exposed on the local juno container. This port is used to communicate with the chain.
//!
//! # Run
//!
//! `cargo run --example local_daemon`

use abstract_app::objects::namespace::Namespace;
use abstract_client::{AbstractClient, Publisher};
use app::{
    contract::{APP_ID, APP_VERSION},
    msg::AppInstantiateMsg,
    AppInterface,
};
use cw_orch::{anyhow, prelude::*, tokio::runtime::Runtime};
use semver::Version;
use speculoos::assert_that;

const LOCAL_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let _version: Version = APP_VERSION.parse().unwrap();
    let runtime = Runtime::new()?;

    let daemon = Daemon::builder()
        .chain(networks::LOCAL_JUNO)
        .mnemonic(LOCAL_MNEMONIC)
        .handle(runtime.handle())
        .build()
        .unwrap();

    let app_namespace = Namespace::from_id(APP_ID)?;

    // Create an [`AbstractClient`]
    let abstract_client: AbstractClient<Daemon> = AbstractClient::new(daemon.clone())?;

    // Get the [`Publisher`] that owns the namespace.
    // If there isn't one, it creates an Account and claims the namespace.
    let publisher: Publisher<_> = abstract_client.publisher_builder(app_namespace).build()?;

    // Ensure the current sender owns the namespace
    if publisher.account().owner()? != daemon.sender() {
        panic!("The current sender can not publish to this namespace. Please use the wallet that owns the Account that owns the Namespace.")
    }

    // Publish the App to the Abstract Platform
    publisher.publish_app::<AppInterface<Daemon>>()?;

    // Install the App on a new account

    let account = abstract_client.account_builder().build()?;
    // Installs the app on the Account
    let app = account.install_app::<AppInterface<_>>(&AppInstantiateMsg { count: 0 }, &[])?;

    // Import app's endpoint function traits for easy interactions.
    use app::{AppExecuteMsgFns, AppQueryMsgFns};
    assert_that!(app.count()?.count).is_equal_to(0);

    // Execute the App
    app.increment()?;

    // Query the App again
    assert_that!(app.count()?.count).is_equal_to(1);

    // Note: the App is installed on a sub-account of the main account!
    assert_ne!(account.id()?, app.account().id()?);

    Ok(())
}
