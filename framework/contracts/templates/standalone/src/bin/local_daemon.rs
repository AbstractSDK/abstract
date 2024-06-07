//! Deploys Abstract and the Standalone module to a local Junod instance. See how to spin up a local chain here: <https://docs.junonetwork.io/developer-guides/junod-local-dev-setup>
//! You can also start a juno container by running `just juno-local`.
//!
//! Ensure the local juno is running before executing this script.
//! Also make sure port 9090 is exposed on the local juno container. This port is used to communicate with the chain.
//!
//! # Run
//!
//! `RUST_LOG=info cargo run --bin --features="daemon-bin" local_daemon --package my-standalone`
use my_standalone::MY_STANDALONE_ID;

use abstract_client::{AbstractClient, Publisher};
use abstract_standalone::{objects::namespace::Namespace, std::standalone};
use cw_orch::{anyhow, prelude::*, tokio::runtime::Runtime};
use my_standalone::{msg::MyStandaloneInstantiateMsg, MyStandaloneInterface};

const LOCAL_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";

fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let runtime = Runtime::new()?;

    let daemon = Daemon::builder()
        .chain(networks::LOCAL_JUNO)
        .mnemonic(LOCAL_MNEMONIC)
        .handle(runtime.handle())
        .build()
        .unwrap();

    let standalone_namespace = Namespace::from_id(MY_STANDALONE_ID)?;

    // Create an [`AbstractClient`]
    // Note: AbstractClient Builder used because Abstract is not yet deployed on the chain
    let abstract_client: AbstractClient<Daemon> =
        AbstractClient::builder(daemon.clone()).build()?;

    // Get the [`Publisher`] that owns the namespace.
    // If there isn't one, it creates an Account and claims the namespace.
    let publisher: Publisher<_> = abstract_client
        .publisher_builder(standalone_namespace)
        .build()?;

    // Ensure the current sender owns the namespace
    if publisher.account().owner()? != daemon.sender() {
        panic!("The current sender can not publish to this namespace. Please use the wallet that owns the Account that owns the Namespace.")
    }

    // Publish the Standalone to the Abstract Platform
    publisher.publish_standalone::<MyStandaloneInterface<Daemon>>()?;

    // Install the Standalone on a new account

    let account = abstract_client.account_builder().build()?;
    // Installs the standalone on the Account
    let standalone = account.install_standalone::<MyStandaloneInterface<_>>(
        &MyStandaloneInstantiateMsg {
            base: standalone::BaseInstantiateMsg {
                ans_host_address: abstract_client.name_service().addr_str()?,
                version_control_address: abstract_client.version_control().addr_str()?,
            },
            count: 0,
        },
        &[],
    )?;

    // Import standalone's endpoint function traits for easy interactions.
    use my_standalone::msg::{MyStandaloneExecuteMsgFns, MyStandaloneQueryMsgFns};
    assert_eq!(standalone.count()?.count, 0);
    // Execute the Standalone
    standalone.increment()?;

    // Query the Standalone again
    assert_eq!(standalone.count()?.count, 1);

    // Note: the Standalone is installed on a sub-account of the main account!
    assert_ne!(account.id()?, standalone.account().id()?);

    Ok(())
}
