#![cfg(test)]

use abstract_app::objects::namespace::Namespace;
use abstract_app::objects::AccountId;

use abstract_client::{AbstractClient, Environment};
use abstract_client::{Application, RemoteAccount};

use abstract_interface::VCQueryFns;
use abstract_std::objects::account::AccountTrace;
use abstract_std::objects::chain_name::ChainName;
use cosmwasm_std::Attribute;
use cw_orch::{anyhow, prelude::*};
use cw_orch_interchain::prelude::*;
use cw_orch_polytone::Polytone;

use crate::setup::ibc_connect_polytone_and_abstract;
use crate::setup::mock_test::logger_test_init;
use crate::{JUNO, STARGAZE};
use ping_pong::contract::APP_ID;
use ping_pong::msg::{AppInstantiateMsg, AppQueryMsg, WinsResponse, PreviousPingPongResponse};
use ping_pong::{AppExecuteMsgFns, AppInterface, AppQueryMsgFns};

#[allow(unused)]
struct PingPong<'a, Env: IbcQueryHandler, IbcEnv: InterchainEnv<Env>> {
    abs_juno: AbstractClient<Env>,
    abs_stargaze: AbstractClient<Env>,
    app: Application<Env, AppInterface<Env>>,
    remote_account: RemoteAccount<'a, Env, IbcEnv>,
}

impl<'a> PingPong<'a, MockBech32, MockBech32InterchainEnv> {
    /// Set up the test environment with two Accounts that has the App installed
    fn setup(
        mock_interchain: &'a MockBech32InterchainEnv,
    ) -> anyhow::Result<PingPong<'a, MockBech32, MockBech32InterchainEnv>> {
        let mock_juno = mock_interchain.chain(JUNO).unwrap();
        let mock_stargaze = mock_interchain.chain(STARGAZE).unwrap();

        let abs_juno = AbstractClient::builder(mock_juno.clone()).build()?;
        let abs_stargaze = AbstractClient::builder(mock_stargaze.clone()).build()?;

        // Deploying polytone on both chains
        Polytone::deploy_on(mock_juno, None)?;
        Polytone::deploy_on(mock_stargaze, None)?;

        ibc_connect_polytone_and_abstract(mock_interchain, JUNO, STARGAZE)?;
        ibc_connect_polytone_and_abstract(mock_interchain, STARGAZE, JUNO)?;

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
        let remote_account = app
            .account()
            .remote_account_builder(mock_interchain, &abs_stargaze)
            .install_app_with_dependencies::<AppInterface<Daemon>>(&AppInstantiateMsg {}, Empty {})?
            .build()?;

        Ok(PingPong {
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
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
    let env = PingPong::setup(&mock_interchain)?;
    let app1 = env.app;

    let mock_stargaze = env.abs_stargaze.environment();

    let pongs = app1.pongs()?;
    assert_eq!(pongs, WinsResponse { wins: 0 });

    let module_addrs = env
        .remote_account
        .module_addresses(vec![APP_ID.to_owned()])?;
    let pongs: WinsResponse = mock_stargaze.query(
        &ping_pong::msg::QueryMsg::from(AppQueryMsg::Pongs {}),
        &module_addrs.modules[0].1,
    )?;
    assert_eq!(pongs, WinsResponse { wins: 0 });

    let app2 = env.remote_account.application::<AppInterface<_>>()?;

    let pongs: WinsResponse =
        app2.query(&ping_pong::msg::QueryMsg::from(AppQueryMsg::Pongs {}))?;

    assert_eq!(pongs, WinsResponse { wins: 0 });
    Ok(())
}

#[test]
fn successful_ping_pong() -> anyhow::Result<()> {
    logger_test_init();

    // Create a sender and mock env
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
    let env = PingPong::setup(&mock_interchain)?;
    let app = env.app;

    // Ensure account created
    let _ensure_created = env
        .abs_stargaze
        .version_control()
        .account_base(AccountId::new(
            1,
            AccountTrace::Remote(vec![ChainName::from_chain_id(JUNO)]),
        )?)?;

    let pp = app.ping_pong(ChainName::from_chain_id(STARGAZE), 4)?;
    let pongs = app.pongs()?;
    assert_eq!(pongs.wins, 4);

    let ibc_wait_response = mock_interchain.wait_ibc(JUNO, pp)?;
    ibc_wait_response.into_result()?;

    let pongs_left_events = ibc_wait_response
        .events()
        .into_iter()
        .map(|ev| {
            ev.attributes
                .into_iter()
                .filter(|attr| attr.key == "pongs_left")
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();
    assert_eq!(
        pongs_left_events,
        vec![
            Attribute::new("pongs_left", "4"),
            Attribute::new("pongs_left", "3"),
            Attribute::new("pongs_left", "2"),
            Attribute::new("pongs_left", "1")
        ]
    );

    let ping_ponged = ibc_wait_response.events().into_iter().find_map(|ev| {
        ev.attributes
            .iter()
            .find(|attr| attr.value == "ping_ponged")
            .cloned()
    });
    assert!(ping_ponged.is_some());

    let pongs = app.pongs()?;
    assert_eq!(pongs.wins, 0);
    let previous_ping_pong = app.previous_ping_pong()?;
    assert_eq!(
        previous_ping_pong,
        PreviousPingPongResponse {
            pongs: Some(4),
            host_chain: Some(ChainName::from_chain_id(STARGAZE))
        }
    );

    let remote_app = env.remote_account.application::<AppInterface<_>>()?;

    let pongs: WinsResponse = remote_app.query(&ping_pong::msg::AppQueryMsg::Pongs {}.into())?;
    assert_eq!(pongs.wins, 0);
    let previous_ping_pong: PreviousPingPongResponse =
        remote_app.query(&ping_pong::msg::AppQueryMsg::PreviousPingPong {}.into())?;
    assert_eq!(
        previous_ping_pong,
        PreviousPingPongResponse {
            pongs: None,
            host_chain: None
        }
    );

    Ok(())
}

#[test]
fn successful_remote_ping_pong() -> anyhow::Result<()> {
    logger_test_init();

    // Create a sender and mock env
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
    let env = PingPong::setup(&mock_interchain)?;
    let app = env.remote_account.application::<AppInterface<_>>()?;

    let remote_ping_pong_response = app.execute(
        &ping_pong::msg::AppExecuteMsg::PingPong {
            opponent_chain: ChainName::from_chain_id(JUNO),
            pongs: 4,
        }
        .into(),
    )?;

    let pongs_left_events = remote_ping_pong_response
        .events()
        .into_iter()
        .map(|ev| {
            ev.attributes
                .into_iter()
                .filter(|attr| attr.key == "pongs_left")
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();
    assert_eq!(
        pongs_left_events,
        vec![
            Attribute::new("pongs_left", "4"),
            Attribute::new("pongs_left", "3"),
            Attribute::new("pongs_left", "2"),
            Attribute::new("pongs_left", "1")
        ]
    );

    let ping_ponged = remote_ping_pong_response
        .events()
        .into_iter()
        .find_map(|ev| {
            ev.attributes
                .iter()
                .find(|attr| attr.value == "ping_ponged")
                .cloned()
        });
    assert!(ping_ponged.is_some());

    let pongs: WinsResponse = app.query(&ping_pong::msg::AppQueryMsg::Pongs {}.into())?;
    assert_eq!(pongs.wins, 0);
    let previous_ping_pong: PreviousPingPongResponse =
        app.query(&ping_pong::msg::AppQueryMsg::PreviousPingPong {}.into())?;
    assert_eq!(
        previous_ping_pong,
        PreviousPingPongResponse {
            pongs: Some(4),
            host_chain: Some(ChainName::from_chain_id(JUNO))
        }
    );

    Ok(())
}

#[test]
fn rematch() -> anyhow::Result<()> {
    logger_test_init();

    // Create a sender and mock env
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);
    let env = PingPong::setup(&mock_interchain)?;
    let app = env.app;
    let remote_app = env.remote_account.application::<AppInterface<_>>()?;

    remote_app.execute(
        &ping_pong::msg::AppExecuteMsg::PingPong {
            opponent_chain: ChainName::from_chain_id(JUNO),
            pongs: 4,
        }
        .into(),
    )?;

    let rematch = app.rematch(env.remote_account.id(), ChainName::from_chain_id(STARGAZE))?;
    let ibc_wait_response = mock_interchain.wait_ibc(JUNO, rematch)?;
    ibc_wait_response.into_result()?;

    let pongs_left_events = ibc_wait_response
        .events()
        .into_iter()
        .map(|ev| {
            ev.attributes
                .into_iter()
                .filter(|attr| attr.key == "pongs_left")
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();
    assert_eq!(
        pongs_left_events,
        vec![
            Attribute::new("pongs_left", "4"),
            Attribute::new("pongs_left", "3"),
            Attribute::new("pongs_left", "2"),
            Attribute::new("pongs_left", "1")
        ]
    );

    let ping_ponged = ibc_wait_response.events().into_iter().find_map(|ev| {
        ev.attributes
            .iter()
            .find(|attr| attr.value == "ping_ponged")
            .cloned()
    });
    assert!(ping_ponged.is_some());

    let pongs = app.pongs()?;
    assert_eq!(pongs.wins, 0);
    let previous_ping_pong = app.previous_ping_pong()?;
    assert_eq!(
        previous_ping_pong,
        PreviousPingPongResponse {
            pongs: Some(4),
            host_chain: Some(ChainName::from_chain_id(STARGAZE))
        }
    );

    // Remote
    let pongs: WinsResponse = remote_app.query(&ping_pong::msg::AppQueryMsg::Pongs {}.into())?;
    assert_eq!(pongs.wins, 0);
    let previous_ping_pong: PreviousPingPongResponse =
        remote_app.query(&ping_pong::msg::AppQueryMsg::PreviousPingPong {}.into())?;
    assert_eq!(
        previous_ping_pong,
        PreviousPingPongResponse {
            pongs: Some(4),
            host_chain: Some(ChainName::from_chain_id(JUNO))
        }
    );

    Ok(())
}
