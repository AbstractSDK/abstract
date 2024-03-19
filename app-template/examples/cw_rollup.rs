//! Deploys Abstract and the App module to a local Junod instance. See how to spin up a local chain here: https://docs.junonetwork.io/developer-guides/junod-local-dev-setup
//! You can also start a juno container by running `just juno-local`.
//!
//! Ensure the local juno is running before executing this script.
//! Also make sure port 9090 is exposed on the local juno container. This port is used to communicate with the chain.
//!
//! # Run
//!
//! `cargo run --example local_daemon`
//!
//! Abstract namespace: 00006162737472616374

/*
Command to run after sync:

docker run -d \
-e NODE_TYPE=light \
-e P2P_NETWORK=mocha \
-p 26650:26650 \
-p 26658:26658 \
-p 26659:26659 \
-v $HOME/.celestia-light-mocha-4/:/home/celestia/.celestia-light-mocha-4/ \
ghcr.io/rollkit/celestia-da:v0.12.10 \
celestia-da light start \
--p2p.network=mocha \
--da.grpc.namespace=00006162737472616374 \
--da.grpc.listen=0.0.0.0:26650 \
--core.ip rpc-mocha.pops.one \
--gateway
*/

use abstract_app::objects::namespace::Namespace;
use abstract_client::{AbstractClient, Publisher};
use app::{
    contract::{APP_ID, APP_VERSION},
    msg::AppInstantiateMsg,
    AppInterface,
};
use cw_orch::{anyhow, daemon::ChainInfo, prelude::*, tokio::runtime::Runtime};
use semver::Version;
use speculoos::assert_that;

const LOCAL_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";

const MY_CW_ROLLUP: ChainInfo = ChainInfo {
    chain_id: "celeswasm",
    gas_denom: "uwasm",
    network_info: networks::NetworkInfo {
        id: "my_rollup",
        pub_address_prefix: "wasm",
        coin_type: 118u32,
    },
    fcd_url: None,
    gas_price: 0.025,
    grpc_urls: &["http://127.0.0.1:9290"],
    kind: networks::ChainKind::Local,
    lcd_url: None,
};

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let _version: Version = APP_VERSION.parse().unwrap();
    let runtime = Runtime::new()?;

    let daemon = Daemon::builder()
        .chain(MY_CW_ROLLUP)
        .mnemonic(LOCAL_MNEMONIC)
        .handle(runtime.handle())
        .build()
        .unwrap();

    let app_namespace = Namespace::from_id(APP_ID)?;

    // Create an [`AbstractClient`]
    let abstract_client: AbstractClient<Daemon> =
        AbstractClient::builder(daemon.clone()).build()?;

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
