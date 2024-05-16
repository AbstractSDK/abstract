#![cfg(test)]

use abstract_app::objects::namespace::Namespace;
use abstract_app::objects::AccountId;

use abstract_client::Application;
use abstract_client::{AbstractClient, Environment};

use abstract_std::objects::chain_name::ChainName;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

use crate::setup::mock_test::logger_test_init;
use crate::setup::{ibc_abstract_setup, ibc_connect_polytone_and_abstract};
use crate::{JUNO, STARGAZE};
use ping_pong::contract::APP_ID;
use ping_pong::msg::{AppInstantiateMsg, PongsResponse};
use ping_pong::{AppExecuteMsgFns, AppInterface, AppQueryMsgFns};

struct PingPong<Env: IbcQueryHandler> {
    abs_juno: AbstractClient<Env>,
    abs_stargaze: AbstractClient<Env>,
    app1: Application<Env, AppInterface<Env>>,
    app2: Application<Env, AppInterface<Env>>,
}

impl PingPong<MockBech32> {
    /// Set up the test environment with two Accounts that has the App installed
    fn setup() -> anyhow::Result<PingPong<MockBech32>> {
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        let mock_juno = mock_interchain.chain(JUNO).unwrap();
        let mock_stargaze = mock_interchain.chain(STARGAZE).unwrap();

        let abs_juno = AbstractClient::builder(mock_juno).build()?;
        let abs_stargaze = AbstractClient::builder(mock_stargaze).build()?;

        let namespace = Namespace::from_id(APP_ID)?;
        // Publish and install on both chains
        let publisher = abs_juno.publisher_builder(namespace.clone()).build()?;
        publisher.publish_app::<AppInterface<_>>()?;
        // TODO: https://github.com/AbstractSDK/abstract/pull/346
        // let app1 = publisher
        //     .account()
        //     .install_app_with_dependencies::<AppInterface<_>>(
        //         &AppInstantiateMsg {},
        //         Empty {},
        //         &[],
        //     )?;
        abs_juno.account_builder().build();

        let publisher = abs_stargaze.publisher_builder(namespace).build()?;
        publisher.publish_app::<AppInterface<_>>()?;
        // let app2 = publisher
        //     .account()
        //     .install_app_with_dependencies::<AppInterface<_>>(
        //         &AppInstantiateMsg {},
        //         Empty {},
        //         &[],
        //     )?;

        Ok(PingPong {
            abs_juno,
            abs_stargaze,
            app1,
            app2,
        })
    }
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    logger_test_init();
    // Create a sender and mock env
    let env = PingPong::setup()?;
    let app1 = env.app1;
    let app2 = env.app2;

    let pongs = app1.pongs()?;
    assert_eq!(pongs, PongsResponse { pongs: 0 });

    let pongs = app2.pongs()?;
    assert_eq!(pongs, PongsResponse { pongs: 0 });
    Ok(())
}

#[test]
fn successful_ping_pong() -> anyhow::Result<()> {
    logger_test_init();
    let env = PingPong::setup()?;
    let app1 = env.app1;
    let app2 = env.app2;

    let r = app1.ping_pong(ChainName::from_chain_id(STARGAZE), 5)?;
    Ok(())
}
