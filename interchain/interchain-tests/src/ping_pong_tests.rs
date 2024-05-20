#![cfg(test)]

use abstract_app::objects::namespace::Namespace;
use abstract_app::objects::AccountId;

use abstract_client::{AbstractClient, Environment};
use abstract_client::{Application, RemoteAccount};

use abstract_interface::{IbcClient, VCQueryFns};
use abstract_std::ibc_client::QueryMsgFns;
use abstract_std::objects::account::AccountTrace;
use abstract_std::objects::chain_name::ChainName;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};
use cw_orch_polytone::Polytone;

use crate::setup::mock_test::logger_test_init;
use crate::setup::{ibc_abstract_setup, ibc_connect_polytone_and_abstract};
use crate::{JUNO, STARGAZE};
use ping_pong::contract::APP_ID;
use ping_pong::msg::{AppInstantiateMsg, AppQueryMsg, PongsResponse};
use ping_pong::{AppExecuteMsgFns, AppInterface, AppQueryMsgFns};

struct PingPong<Env: IbcQueryHandler, IbcEnv: InterchainEnv<Env>> {
    interchain: IbcEnv,
    abs_juno: AbstractClient<Env>,
    abs_stargaze: AbstractClient<Env>,
    app: Application<Env, AppInterface<Env>>,
    remote_account: RemoteAccount<Env>,
}

impl PingPong<MockBech32, MockBech32InterchainEnv> {
    /// Set up the test environment with two Accounts that has the App installed
    fn setup() -> anyhow::Result<PingPong<MockBech32, MockBech32InterchainEnv>> {
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        let mock_juno = mock_interchain.chain(JUNO).unwrap();
        let mock_stargaze = mock_interchain.chain(STARGAZE).unwrap();

        let abs_juno = AbstractClient::builder(mock_juno.clone()).build()?;
        let abs_stargaze = AbstractClient::builder(mock_stargaze.clone()).build()?;

        // Deploying polytone on both chains
        Polytone::deploy_on(mock_juno, None)?;
        Polytone::deploy_on(mock_stargaze, None)?;

        ibc_connect_polytone_and_abstract(&mock_interchain, JUNO, STARGAZE)?;
        ibc_connect_polytone_and_abstract(&mock_interchain, STARGAZE, JUNO)?;

        let namespace = Namespace::from_id(APP_ID)?;
        // Publish and install on both chains
        let publisher_juno = abs_juno.publisher_builder(namespace.clone()).build()?;
        publisher_juno.publish_app::<AppInterface<_>>()?;
        let app = publisher_juno
            .account()
            .install_app_with_dependencies::<AppInterface<_>>(
                &AppInstantiateMsg {},
                Empty {},
                &[],
            )?;

        let publisher_stargaze = abs_stargaze.publisher_builder(namespace).build()?;
        publisher_stargaze.publish_app::<AppInterface<_>>()?;

        publisher_juno.account().set_ibc_status(true)?;
        let (remote_account, account_response) = abs_stargaze
            .account_builder()
            .remote_account(&app.account())
            .install_app::<AppInterface<Daemon>>(&AppInstantiateMsg {})?
            .build_remote()?;
        mock_interchain
            .wait_ibc(JUNO, account_response)?
            .into_result()?;

        Ok(PingPong {
            interchain: mock_interchain,
            abs_juno,
            abs_stargaze,
            app,
            remote_account,
        })
    }
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    logger_test_init();

    // Create a sender and mock env
    let env = PingPong::setup()?;
    let app1 = env.app;

    let mock_stargaze = env.abs_stargaze.environment();

    let pongs = app1.pongs()?;
    assert_eq!(pongs, PongsResponse { pongs: 0 });

    let module_addrs = env
        .remote_account
        .module_addresses(vec![APP_ID.to_owned()])?;
    let pongs: PongsResponse = mock_stargaze.query(
        &ping_pong::msg::QueryMsg::from(AppQueryMsg::Pongs {}),
        &module_addrs.modules[0].1,
    )?;
    assert_eq!(pongs, PongsResponse { pongs: 0 });
    Ok(())
}

#[test]
fn successful_ping_pong() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    logger_test_init();

    let env = PingPong::setup()?;
    let app1 = env.app;

    // Ensure account created
    let _ensure_created = env
        .abs_stargaze
        .version_control()
        .account_base(AccountId::new(
            1,
            AccountTrace::Remote(vec![ChainName::from_chain_id(JUNO)]),
        )?)?;

    let pp = app1.ping_pong(ChainName::from_chain_id(STARGAZE), 1)?;

    // let pongs = dbg!(app1.pongs())?;
    env.interchain.wait_ibc(JUNO, pp)?.into_result()?;

    // let pongs = dbg!(app1.pongs())?;

    Ok(())
}
