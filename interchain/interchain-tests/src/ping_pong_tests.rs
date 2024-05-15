use abstract_app::objects::namespace::Namespace;
use abstract_app::objects::AccountId;

use abstract_client::Application;
use abstract_client::{AbstractClient, Environment};
use abstract_cw_orch_polytone::Polytone;

// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

use abstract_interchain_tests::setup::ibc_connect_polytone_and_abstract;
use ping_pong::contract::APP_ID;
use ping_pong::msg::{AppInstantiateMsg, PongsResponse};
use ping_pong::{AppExecuteMsgFns, AppInterface, AppQueryMsgFns};

struct PingPong<Env: CwEnv> {
    abs: AbstractClient<Env>,
    app1: Application<Env, AppInterface<Env>>,
    app2: Application<Env, AppInterface<Env>>,
}

impl<Env: CwEnv> PingPong<Env> {
    /// Set up the test environment with two Accounts that has the App installed
    fn setup(env: Env) -> anyhow::Result<PingPong<Env>> {
        let namespace = Namespace::from_id(APP_ID)?;

        // You can set up Abstract with a builder.
        let abs_client = AbstractClient::builder(env.clone()).build()?;

        // Publish both the client and the server
        let publisher = abs_client.publisher_builder(namespace).build()?;
        publisher.publish_app::<AppInterface<_>>()?;

        let app1 = publisher
            .account()
            .install_app_with_dependencies::<AppInterface<_>>(
                &AppInstantiateMsg {},
                Empty {},
                &[],
            )?;

        let app2 = publisher
            .account()
            .install_app_with_dependencies::<AppInterface<_>>(
                &AppInstantiateMsg {},
                Empty {},
                &[],
            )?;

        Polytone::deploy_on(env, None)?;

        Ok(PingPong {
            abs: abs_client,
            app1,
            app2,
        })
    }
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Create a sender and mock env
    let mock = MockBech32::new("mock");
    let env = PingPong::setup(mock)?;
    let app1 = env.app1;

    let pongs = app1.pongs()?;
    assert_eq!(pongs, PongsResponse { pongs: 0 });
    Ok(())
}

#[test]
fn successful_ping_pong() -> anyhow::Result<()> {
    // Create a sender and mock env
    let mock = MockBech32InterchainEnv::new("mock");
    let env = PingPong::setup(mock)?;
    let app1 = env.app1;

    let pongs = app1.ping_pong()?;
    assert_eq!(pongs, PongsResponse { pongs: 0 });
    Ok(())
}
